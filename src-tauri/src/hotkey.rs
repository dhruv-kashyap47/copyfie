//! Global shortcut registration and transactional rebinding.
//!
//! Rules (PLAN §6/§9):
//! - The default is `CommandOrControl+Shift+Space`.
//! - A failed registration is never silently replaced by another chord. The
//!   caller shows the blocking setup UI and the user picks a replacement.
//! - Rebinding is transactional: the new shortcut must register **before** the
//!   old one is unregistered, so a conflict leaves the previous shortcut working.

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub struct HotkeyManager {
    state: Mutex<HotkeyState>,
    initialized: AtomicBool,
}

struct HotkeyState {
    configured: String,
    registered: Option<Shortcut>,
}

impl HotkeyManager {
    pub fn new(configured: String) -> Self {
        Self {
            state: Mutex::new(HotkeyState {
                configured,
                registered: None,
            }),
            initialized: AtomicBool::new(false),
        }
    }

    /// Called after the startup registration attempt, so the UI can tell
    /// "not registered yet" apart from "registration failed".
    pub fn mark_initialized(&self) {
        self.initialized.store(true, Ordering::Relaxed);
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }

    pub fn configured(&self) -> String {
        self.lock().configured.clone()
    }

    pub fn is_registered(&self) -> bool {
        self.lock().registered.is_some()
    }

    /// Register `accelerator`, replacing the current shortcut only on success.
    ///
    /// Parsing and availability errors are returned to the caller; the previous
    /// registration is untouched.
    pub fn try_register(&self, app: &AppHandle, accelerator: &str) -> Result<(), String> {
        let trimmed = accelerator.trim();
        if trimmed.is_empty() {
            return Err("record a shortcut first".to_string());
        }

        let shortcut = Shortcut::from_str(trimmed)
            .map_err(|error| format!("unsupported shortcut: {error}"))?;

        let mut state = self.lock();

        if state.registered.as_ref() == Some(&shortcut) {
            state.configured = trimmed.to_string();
            return Ok(());
        }

        app.global_shortcut()
            .register(shortcut)
            .map_err(|error| format!("{trimmed} is already in use or unavailable: {error}"))?;

        if let Some(previous) = state.registered.replace(shortcut) {
            if let Err(error) = app.global_shortcut().unregister(previous) {
                log::warn!(
                    target: "copyfie::hotkey",
                    "could not unregister the previous shortcut: {error}"
                );
            }
        }

        state.configured = trimmed.to_string();
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HotkeyState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
