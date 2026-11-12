//! Shared application state managed by Tauri.
//!
//! The store handle is cloneable; nothing here holds a lock. Settings join this
//! struct in M1-10, licence state in M2-13.

use std::sync::Arc;

use crate::capture::Suppressor;
use crate::paste::PasteService;
use crate::store::StoreHandle;

pub struct AppState {
    pub store: StoreHandle,
    /// Shared with the capture pipeline so writes from paste/copy never echo.
    pub suppressor: Arc<Suppressor>,
    /// Focus capture/restore + synthetic input for paste.
    pub paste: Arc<PasteService>,
}

impl AppState {
    pub fn new(store: StoreHandle, suppressor: Arc<Suppressor>, paste: Arc<PasteService>) -> Self {
        Self {
            store,
            suppressor,
            paste,
        }
    }
}
