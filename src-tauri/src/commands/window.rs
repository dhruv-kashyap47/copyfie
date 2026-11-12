//! Window commands and the lazily created settings window.
//!
//! Palette lifecycle lives in `crate::palette` (pre-warm, hybrid idle destroy,
//! cold open, position, focus). This module only exposes commands and the
//! settings window.

use tauri::{AppHandle, Manager, WebviewWindow};

use crate::palette::{force_foreground, PaletteManager, BROWSER_ARGS};

#[tauri::command]
pub fn hide_palette(window: WebviewWindow, app: AppHandle) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())?;
    if let Some(palette) = app.try_state::<PaletteManager>() {
        palette.after_hide(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn palette_show(app: AppHandle) -> Result<(), String> {
    show_palette(&app)
}

#[tauri::command]
pub fn palette_painted(open_id: u64, app: AppHandle) {
    app.state::<PaletteManager>().painted(open_id);
}

pub fn show_palette(app: &AppHandle) -> Result<(), String> {
    app.state::<PaletteManager>().show(app)
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

            // Must match the palette's browser args or WebView2 returns
            // ERROR_INVALID_STATE when creating the second webview.
            #[cfg(target_os = "windows")]
            let builder = builder.additional_browser_args(BROWSER_ARGS);

            builder.build().map_err(|error| {
                log::error!(
                    target: "copyfie::window",
                    "could not create the settings window: {error}"
                );
                error.to_string()
            })?
        }
    };

    // Hide the palette only after the settings window exists.
    if let Some(palette) = app.get_webview_window("main") {
        let _ = palette.hide();
        if let Some(manager) = app.try_state::<PaletteManager>() {
            manager.after_hide(app);
        }
    }

    window.show().map_err(|error| error.to_string())?;
    force_foreground(&window)?;

    Ok(())
}
