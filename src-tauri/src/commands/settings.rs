//! Settings commands.
//!
//! M1-10 exposes the minimal settings surface: theme, accent, retention,
//! launch-at-login, pause capture, and the global shortcut. The full settings
//! surface (blocklist, image cap, updates) lands with M2-09.

use copyfie_core::Retention;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub app_version: String,
    pub theme: String,
    pub accent: String,
    pub retention_days: u32,
    pub capture_paused: bool,
    pub launch_at_login: bool,
    pub hotkey: String,
    pub hotkey_registered: bool,
    pub palette_ready: String,
    pub palette_idle_minutes: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsChanged {
    pub theme: String,
    pub accent: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub theme: String,
    pub accent: String,
    pub retention_days: u32,
    pub capture_paused: bool,
    pub launch_at_login: bool,
    pub hotkey: String,
    pub palette_ready: String,
    pub palette_idle_minutes: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub configured: String,
    pub registered: bool,
    /// False until the startup registration attempt has run.
    pub initialized: bool,
}

fn status_of(state: &AppState) -> HotkeyStatus {
    HotkeyStatus {
        configured: state.hotkey.configured(),
        registered: state.hotkey.is_registered(),
        initialized: state.hotkey.is_initialized(),
    }
}

fn view(app: &AppHandle, state: &AppState) -> SettingsView {
    let settings = state.settings.load();
    SettingsView {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        theme: settings.theme,
        accent: settings.accent,
        retention_days: settings.retention_days,
        capture_paused: settings.capture_paused,
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
        hotkey: settings.hotkey,
        hotkey_registered: state.hotkey.is_registered(),
        palette_ready: settings.palette_ready,
        palette_idle_minutes: settings.palette_idle_minutes,
    }
}

#[tauri::command]
pub fn hotkey_status(state: State<'_, AppState>) -> HotkeyStatus {
    status_of(&state)
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
    Ok(status_of(&state))
}

#[tauri::command]
pub fn settings_get(app: AppHandle, state: State<'_, AppState>) -> SettingsView {
    view(&app, &state)
}

/// Apply and persist the settings form.
///
/// The hotkey is registered before anything is written, so a conflict leaves
/// every value untouched. Remaining values are clamped server-side.
#[tauri::command]
pub fn settings_save(
    input: SettingsInput,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SettingsView, String> {
    let mut settings = state.settings.load();
    let hotkey = input.hotkey.trim().to_string();

    if hotkey != settings.hotkey {
        state.hotkey.try_register(&app, &hotkey)?;
    }

    settings.hotkey = hotkey;
    settings.theme = input.theme;
    settings.accent = input.accent;
    settings.retention_days = input.retention_days;
    settings.capture_paused = input.capture_paused;
    settings.palette_ready = input.palette_ready;
    settings.palette_idle_minutes = input.palette_idle_minutes;

    let settings = settings.sanitized();
    state.settings.save(&settings)?;

    let autolaunch = app.autolaunch();
    let enabled = autolaunch.is_enabled().unwrap_or(false);
    if input.launch_at_login != enabled {
        let result = if input.launch_at_login {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        if let Err(error) = result {
            log::warn!(target: "copyfie::settings", "could not change autostart: {error}");
        }
    }

    {
        let config = state.capture_config.clone();
        let mut config = config.write().unwrap_or_else(|poison| poison.into_inner());
        config.retention = Retention::from_days(settings.retention_days);
        config.paused = settings.capture_paused;
    }

    if let Some(palette) = app.try_state::<crate::palette::PaletteManager>() {
        palette.apply_settings(&app, &settings);
    }

    let window_theme = match settings.theme.as_str() {
        "light" => Some(tauri::Theme::Light),
        "dark" => Some(tauri::Theme::Dark),
        _ => None,
    };
    for label in ["main", "settings"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.set_theme(window_theme);
        }
    }

    let _ = app.emit(
        "settings:changed",
        SettingsChanged {
            theme: settings.theme.clone(),
            accent: settings.accent.clone(),
        },
    );

    log::info!(
        target: "copyfie::settings",
        "settings saved (theme={}, retention={}, paused={})",
        settings.theme,
        settings.retention_days,
        settings.capture_paused
    );

    Ok(view(&app, &state))
}
