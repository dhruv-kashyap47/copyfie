//! Window commands and palette placement.
//!
//! The palette is created hidden at startup (pre-warmed), then positioned on
//! the monitor under the cursor **before** it is shown. Positioning a visible
//! window causes the flicker/DPI bugs the plan calls out.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

/// Event the palette listens for to reset and refocus.
pub const PALETTE_OPEN: &str = "palette:open";

static OPEN_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteOpen {
    pub open_id: u64,
}

#[tauri::command]
pub fn hide_palette(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn palette_show(app: AppHandle) -> Result<(), String> {
    show_palette(&app)
}

/// Open (or focus) the lazily created settings window.
///
/// Async on purpose: WebView2 window creation must run off the main thread and
/// dispatch to the event loop. A synchronous command deadlocks here.
#[tauri::command]
pub async fn open_settings(app: AppHandle) -> Result<(), String> {
    show_settings(&app)
}

pub fn show_settings(app: &AppHandle) -> Result<(), String> {
    log::info!(target: "copyfie::window", "opening settings window");
    let existing = app.get_webview_window("settings");

    let window = match existing {
        Some(window) => window,
        None => {
            let theme = app
                .try_state::<crate::state::AppState>()
                .map(|state| state.settings.load().theme)
                .unwrap_or_else(|| "system".to_string());

            let window_theme = match theme.as_str() {
                "light" => Some(tauri::Theme::Light),
                "dark" => Some(tauri::Theme::Dark),
                _ => None,
            };

            let builder = tauri::WebviewWindowBuilder::new(
                app,
                "settings",
                tauri::WebviewUrl::App("settings.html".into()),
            )
            .title("Copyfie Settings")
            .inner_size(560.0, 640.0)
            .resizable(false)
            .center()
            .theme(window_theme)
            .on_page_load(|_webview, payload| {
                log::info!(
                    target: "copyfie::window",
                    "settings page {} ({:?})",
                    payload.url(),
                    payload.event()
                );
            });

            // WebView2 shares one environment per data directory. Its browser
            // arguments must match the palette's, or creating a second webview
            // fails with HRESULT 0x8007139F (ERROR_INVALID_STATE).
            #[cfg(target_os = "windows")]
            let builder = builder.additional_browser_args(
                "--disable-background-timer-throttling \
                 --disable-renderer-backgrounding \
                 --disable-backgrounding-occluded-windows",
            );

            let window = builder.build().map_err(|error| {
                log::error!(
                    target: "copyfie::window",
                    "could not create the settings window: {error}"
                );
                error.to_string()
            })?;

            log::info!(
                target: "copyfie::window",
                "settings window url={:?}",
                window.url().map(|url| url.to_string())
            );

            window
        }
    };

    // Hide the palette only after the settings window exists.
    if let Some(palette) = app.get_webview_window("main") {
        let _ = palette.hide();
    }

    window.show().map_err(|error| error.to_string())?;
    force_foreground(&window)?;

    Ok(())
}

/// Position, show, focus, and notify the palette.
pub fn show_palette(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "palette window is not available".to_string())?;

    // Capture the target only while the palette is hidden, so showing it twice
    // in a row never records the palette as its own paste target.
    if !window.is_visible().unwrap_or(false) {
        if let Some(state) = app.try_state::<crate::state::AppState>() {
            state.paste.remember_target();
        }
    }

    place_on_cursor_monitor(&window, app).map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    force_foreground(&window)?;

    let open_id = OPEN_ID.fetch_add(1, Ordering::Relaxed) + 1;
    app.emit(PALETTE_OPEN, PaletteOpen { open_id })
        .map_err(|error| error.to_string())?;

    log::info!(
        target: "copyfie::window",
        "palette shown (open_id={open_id})"
    );

    Ok(())
}

/// Windows refuses `SetForegroundWindow` when the caller is not the foreground
/// process. Attaching to the foreground thread's input queue makes the call
/// succeed for a user-triggered show (hotkey / `--show`).
#[cfg(target_os = "windows")]
fn force_foreground(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
    };

    let hwnd = window.hwnd().map_err(|error| error.to_string())?.0 as isize;
    let hwnd = hwnd as *mut core::ffi::c_void;

    // SAFETY: handles come from Win32 and are only passed back to Win32. The
    // input-queue attachment is always detached on the same thread.
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

    window.set_focus().map_err(|error| error.to_string())
}

#[cfg(not(target_os = "windows"))]
fn force_foreground(window: &WebviewWindow) -> Result<(), String> {
    window.set_focus().map_err(|error| error.to_string())
}

/// Center horizontally on the cursor's monitor and sit ~18% down the work
/// area. All coordinates are physical pixels, so mixed-DPI setups position
/// correctly without manual scaling.
fn place_on_cursor_monitor(window: &WebviewWindow, app: &AppHandle) -> tauri::Result<()> {
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
