//! Shared application state managed by Tauri.
//!
//! The store handle is cloneable; nothing here holds a lock. Settings, the
//! capture pipeline's data directory, and licence state join this struct in
//! M1-04, M1-10, and M2-13.

use crate::store::StoreHandle;

pub struct AppState {
    pub store: StoreHandle,
}

impl AppState {
    pub fn new(store: StoreHandle) -> Self {
        Self { store }
    }
}
