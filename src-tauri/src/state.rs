//! Shared application state managed by Tauri.
//!
//! All handles are cheap to clone; nothing here holds a long-lived lock except
//! the capture config and the hotkey state, both short critical sections.

use std::sync::{Arc, RwLock};

use crate::capture::{CaptureConfig, Suppressor};
use crate::hotkey::HotkeyManager;
use crate::paste::PasteService;
use crate::settings::SettingsStore;
use crate::store::StoreHandle;

pub struct AppState {
    pub store: StoreHandle,
    /// Shared with the capture pipeline so writes from paste/copy never echo.
    pub suppressor: Arc<Suppressor>,
    /// Focus capture/restore + synthetic input for paste.
    pub paste: Arc<PasteService>,
    /// Read by capture and the tray menu (pause/resume).
    pub capture_config: Arc<RwLock<CaptureConfig>>,
    pub hotkey: Arc<HotkeyManager>,
    pub settings: Arc<SettingsStore>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: StoreHandle,
        suppressor: Arc<Suppressor>,
        paste: Arc<PasteService>,
        capture_config: Arc<RwLock<CaptureConfig>>,
        hotkey: Arc<HotkeyManager>,
        settings: Arc<SettingsStore>,
    ) -> Self {
        Self {
            store,
            suppressor,
            paste,
            capture_config,
            hotkey,
            settings,
        }
    }
}
