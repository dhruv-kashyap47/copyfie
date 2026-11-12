//! Window commands.
//!
//! All window control goes through Rust: the webview has no window
//! permissions. Pre-warm and positioning land in M1-07.

use tauri::WebviewWindow;

#[tauri::command]
pub fn hide_palette(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}
