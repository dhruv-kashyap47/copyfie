//! Palette lifecycle: pre-warm, warm/cold show, idle destruction.
//!
//! Hybrid policy (default):
//! - the palette is created hidden at startup and stays warm while the user is
//!   active;
//! - after `palette_idle_minutes` hidden, the webview is destroyed so only the
//!   core process remains resident;
//! - the next hotkey creates it cold, injects a bootstrap snapshot (top items,
//!   theme), positions, shows, and focuses it.
//!
//! `palette_ready = always` never destroys; `never` creates on hotkey and
//! destroys on hide.
//!
//! Window creation must never run on the main thread after the event loop is
//! running (WebView2 window creation would deadlock). Cold opens and settings
//! changes that pre-warm run on worker threads.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, Theme, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use copyfie_core::ListQuery;

use crate::settings::Settings;
use crate::state::AppState;

pub const PALETTE_OPEN: &str = "palette:open";

/// WebView2 browser arguments shared by every webview in the process.
///
/// The environment is keyed by data directory and its options must match, or a
/// second webview fails with `ERROR_INVALID_STATE (0x8007139F)`.
pub(crate) const BROWSER_ARGS: &str = "--disable-background-timer-throttling \
     --disable-renderer-backgrounding \
     --disable-backgrounding-occluded-windows \
     --disable-background-networking \
     --disable-component-update \
     --disable-default-apps \
     --disable-sync \
     --disable-client-side-phishing-detection \
     --metrics-recording-only \
     --no-first-run \
     --disable-gpu \
     --disable-breakpad";

pub struct PaletteManager {
    next_id: AtomicU64,
    open_attempt: Mutex<Option<OpenAttempt>>,
    last_hidden: Mutex<Option<Instant>>,
}

struct OpenAttempt {
    id: u64,
    started: Instant,
    cold: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaletteOpen {
    open_id: u64,
    cold: bool,
}

impl PaletteManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
            open_attempt: Mutex::new(None),
            last_hidden: Mutex::new(None),
        }
    }

    /// Create the hidden palette window (Auto/Always). Safe from setup.
    pub fn prewarm(&self, app: &AppHandle) {
        if app.get_webview_window("main").is_some() {
            return;
        }

        match self.ensure_window(app, 0, false) {
            Ok(_) => log::info!(target: "copyfie::palette", "palette pre-warmed"),
            Err(error) => log::warn!(
                target: "copyfie::palette",
                "pre-warm failed; the next hotkey will create it cold: {error}"
            ),
        }
    }

    /// Show the palette, warm if it exists, otherwise cold on a worker thread.
    pub fn show(&self, app: &AppHandle) -> Result<(), String> {
        log::debug!(target: "copyfie::palette", "show requested");

        if let Some(window) = app.get_webview_window("main") {
            return self.show_existing(app, &window);
        }

        log::debug!(target: "copyfie::palette", "no palette window; starting cold open");

        let app = app.clone();
        std::thread::Builder::new()
            .name("copyfie-palette-cold".to_string())
            .spawn(move || {
                if let Err(error) = cold_open(&app) {
                    log::error!(target: "copyfie::palette", "cold open failed: {error}");
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn show_existing(&self, app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
        if !window.is_visible().unwrap_or(false) {
            remember_paste_target(app);
        }

        let open_id = self.begin_open(false);

        log::debug!(target: "copyfie::palette", "positioning warm palette (open_id={open_id})");
        place_on_cursor_monitor(window, app).map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
        log::debug!(target: "copyfie::palette", "warm palette shown, focusing");
        force_foreground(window)?;

        self.clear_hidden();
        app.emit(
            PALETTE_OPEN,
            PaletteOpen {
                open_id,
                cold: false,
            },
        )
        .map_err(|error| error.to_string())?;

        log::info!(
            target: "copyfie::palette",
            "palette shown (open_id={open_id}, cold=false)"
        );
        Ok(())
    }

    /// Record that the palette was hidden. `never` destroys immediately;
    /// `auto` starts the idle countdown; `always` keeps it.
    pub fn after_hide(&self, app: &AppHandle) {
        if read_mode(app) == "never" {
            self.destroy(app);
            return;
        }

        *self
            .last_hidden
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(Instant::now());
    }

    /// Destroy the hidden webview, leaving only the core process resident.
    pub fn destroy(&self, app: &AppHandle) {
        let Some(window) = app.get_webview_window("main") else {
            return;
        };

        if window.is_visible().unwrap_or(false) {
            return;
        }

        // Do not race a cold open that has created the window but not shown it.
        if self
            .open_attempt
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .is_some()
        {
            return;
        }

        match window.destroy() {
            Ok(()) => log::info!(
                target: "copyfie::palette",
                "palette webview destroyed; core stays resident"
            ),
            Err(error) => log::warn!(
                target: "copyfie::palette",
                "could not destroy the palette webview: {error}"
            ),
        }
        self.clear_hidden();
    }

    /// Called by the idle monitor. `always` is a no-op; `never` destroys a
    /// hidden palette; `auto` destroys after the configured idle period.
    pub fn enforce_idle(&self, app: &AppHandle, settings: &Settings) {
        let Some(window) = app.get_webview_window("main") else {
            return;
        };
        if window.is_visible().unwrap_or(false) {
            return;
        }
        if self
            .open_attempt
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .is_some()
        {
            return;
        }

        match settings.palette_ready.as_str() {
            "always" => {}
            "never" => self.destroy(app),
            _ => {
                let mut hidden = self
                    .last_hidden
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner());
                let Some(since) = *hidden else {
                    *hidden = Some(Instant::now());
                    return;
                };
                let timeout =
                    Duration::from_secs(u64::from(settings.palette_idle_minutes.max(1)) * 60);
                if since.elapsed() >= timeout {
                    drop(hidden);
                    self.destroy(app);
                }
            }
        }
    }

    /// Apply a settings change without restarting.
    pub fn apply_settings(&self, app: &AppHandle, settings: &Settings) {
        match settings.palette_ready.as_str() {
            "never" => {
                if let Some(window) = app.get_webview_window("main") {
                    if !window.is_visible().unwrap_or(false) {
                        self.destroy(app);
                    }
                }
            }
            _ => {
                if app.get_webview_window("main").is_none() {
                    let app = app.clone();
                    std::thread::spawn(move || {
                        if let Some(palette) = app.try_state::<PaletteManager>() {
                            if let Err(error) = palette.ensure_window(&app, 0, false) {
                                log::warn!(
                                    target: "copyfie::palette",
                                    "could not pre-warm after settings change: {error}"
                                );
                            }
                        }
                    });
                } else {
                    // Starting the idle countdown from now is safe even if the
                    // window is currently visible (enforce_idle skips visible).
                    if let Some(window) = app.get_webview_window("main") {
                        if !window.is_visible().unwrap_or(false) {
                            *self
                                .last_hidden
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner()) =
                                Some(Instant::now());
                        }
                    }
                }
            }
        }
    }

    /// Called by the frontend after the first paint of a show.
    pub fn painted(&self, open_id: u64) {
        let attempt = self
            .open_attempt
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take();

        if let Some(attempt) = attempt {
            if attempt.id == open_id {
                log::info!(
                    target: "copyfie::perf",
                    "palette_open_ms={} cold={}",
                    attempt.started.elapsed().as_millis(),
                    attempt.cold
                );
            }
        }
    }

    fn begin_open(&self, cold: bool) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        *self
            .open_attempt
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(OpenAttempt {
            id,
            started: Instant::now(),
            cold,
        });
        id
    }

    fn cancel_open(&self, open_id: u64) {
        let mut attempt = self
            .open_attempt
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if attempt.as_ref().is_some_and(|value| value.id == open_id) {
            *attempt = None;
        }
    }

    fn clear_hidden(&self) {
        *self
            .last_hidden
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = None;
    }

    /// Build the hidden palette window with a bootstrap snapshot.
    fn ensure_window(
        &self,
        app: &AppHandle,
        open_id: u64,
        cold: bool,
    ) -> Result<WebviewWindow, String> {
        if let Some(window) = app.get_webview_window("main") {
            return Ok(window);
        }

        let state = app
            .try_state::<AppState>()
            .ok_or_else(|| "application state is not ready".to_string())?;
        let settings = state.settings.load();
        let items = state
            .store
            .call_blocking(|db| {
                db.list(&ListQuery {
                    limit: 50,
                    ..ListQuery::default()
                })
            })
            .unwrap_or_default();

        let bootstrap = serde_json::json!({
            "openId": open_id,
            "cold": cold,
            "theme": settings.theme,
            "accent": settings.accent,
            "items": items,
        });
        let encoded = serde_json::to_string(&bootstrap)
            .map_err(|error| error.to_string())?
            .replace('<', "\\u003c");
        let script = format!("window.__COPYFIE_BOOTSTRAP__ = {encoded};");

        let window_theme = match settings.theme.as_str() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        };

        let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .title("Copyfie")
            .inner_size(680.0, 480.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .focused(false)
            .shadow(true)
            .theme(window_theme)
            .initialization_script(&script);

        // The glass shell is drawn in CSS; the window itself must be
        // transparent. On Windows, skipping the redirection bitmap avoids the
        // white flash while the transparent webview first paints.
        #[cfg(target_os = "windows")]
        let builder = builder
            .no_redirection_bitmap(true)
            .additional_browser_args(BROWSER_ARGS);

        builder.build().map_err(|error| {
            log::error!(target: "copyfie::palette", "could not create the palette window: {error}");
            error.to_string()
        })
    }
}

impl Default for PaletteManager {
    fn default() -> Self {
        Self::new()
    }
}

fn cold_open(app: &AppHandle) -> Result<(), String> {
    let palette = app
        .try_state::<PaletteManager>()
        .ok_or_else(|| "palette manager is not ready".to_string())?;

    let open_id = palette.begin_open(true);
    let window = match palette.ensure_window(app, open_id, true) {
        Ok(window) => window,
        Err(error) => {
            palette.cancel_open(open_id);
            return Err(error);
        }
    };

    remember_paste_target(app);
    place_on_cursor_monitor(&window, app).map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    force_foreground(&window)?;
    palette.clear_hidden();

    app.emit(
        PALETTE_OPEN,
        PaletteOpen {
            open_id,
            cold: true,
        },
    )
    .map_err(|error| error.to_string())?;

    log::info!(
        target: "copyfie::palette",
        "palette shown (open_id={open_id}, cold=true)"
    );
    Ok(())
}

pub fn start_idle_monitor(app: AppHandle) {
    std::thread::Builder::new()
        .name("copyfie-palette-idle".to_string())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(10));

            let (Some(state), Some(palette)) = (
                app.try_state::<AppState>(),
                app.try_state::<PaletteManager>(),
            ) else {
                continue;
            };

            let settings = state.settings.load();
            palette.enforce_idle(&app, &settings);
        })
        .expect("failed to spawn the palette idle monitor");
}

fn read_mode(app: &AppHandle) -> String {
    app.try_state::<AppState>()
        .map(|state| state.settings.load().palette_ready)
        .unwrap_or_else(|| "auto".to_string())
}

fn remember_paste_target(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        state.paste.remember_target();
    }
}

/// Center horizontally on the cursor's monitor and sit ~18% down the work
/// area. Physical pixels keep mixed-DPI setups correct.
pub(crate) fn place_on_cursor_monitor(
    window: &WebviewWindow,
    app: &AppHandle,
) -> tauri::Result<()> {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return Ok(());
    };

    let work_area = monitor.work_area();
    let window_size = window.outer_size()?;

    let width = work_area.size.width as i32;
    let height = work_area.size.height as i32;
    let window_width = window_size.width as i32;

    let x = work_area.position.x + (width - window_width).max(0) / 2;
    let y = work_area.position.y + (height as f64 * 0.18) as i32;

    window.set_position(PhysicalPosition::new(x, y))
}

/// Windows refuses `SetForegroundWindow` for background processes. Attach to
/// the foreground thread's input queue for a user-triggered show.
pub(crate) fn force_foreground(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
        };

        let hwnd = window.hwnd().map_err(|error| error.to_string())?.0 as isize;
        let hwnd = hwnd as *mut core::ffi::c_void;

        // SAFETY: handles come from Win32 and are only passed back to Win32.
        unsafe {
            let foreground = GetForegroundWindow();
            let foreground_thread = GetWindowThreadProcessId(foreground, std::ptr::null_mut());
            let current_thread = GetCurrentThreadId();

            let attached = foreground_thread != 0
                && foreground_thread != current_thread
                && AttachThreadInput(current_thread, foreground_thread, 1) != 0;

            BringWindowToTop(hwnd);
            SetForegroundWindow(hwnd);

            if attached {
                AttachThreadInput(current_thread, foreground_thread, 0);
            }
        }
    }

    window.set_focus().map_err(|error| error.to_string())
}
