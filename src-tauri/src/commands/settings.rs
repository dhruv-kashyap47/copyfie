//! Settings commands.
//!
//! M1-09 exposes only the shortcut state and rebind action. The full settings
//! surface (retention, theme, blocklist, autostart toggle, pause) lands with
//! M1-10/M2-09.

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub configured: String,
    pub registered: bool,
    /// False until the startup registration attempt has run.
    pub initialized: bool,
}

fn status(state: &AppState) -> HotkeyStatus {
    HotkeyStatus {
        configured: state.hotkey.configured(),
        registered: state.hotkey.is_registered(),
        initialized: state.hotkey.is_initialized(),
    }
}

#[tauri::command]
pub fn hotkey_status(state: State<'_, AppState>) -> HotkeyStatus {
    status(&state)
}

/// Transactionally register and persist a new shortcut.
///
/// The old shortcut stays active if the new one is unavailable.
#[tauri::command]
pub fn set_hotkey(
    accelerator: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<HotkeyStatus, String> {
    let trimmed = accelerator.trim().to_string();
    if trimmed.is_empty() {
        return Err("record a shortcut first".to_string());
    }

    state.hotkey.try_register(&app, &trimmed)?;

    if let Err(error) = state.settings.save_hotkey(&trimmed) {
        log::warn!(
            target: "copyfie::hotkey",
            "shortcut is active for this session but could not be saved: {error}"
        );
        return Err(error);
    }

    log::info!(target: "copyfie::hotkey", "shortcut registered");
    Ok(status(&state))
}
