//! IPC commands.
//!
//! Commands are thin: validate, ask the store actor, map errors to strings.
//! Entitlement checks are added in one place when the licence module lands.

pub mod history;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    core_version: &'static str,
    identifier: &'static str,
}

/// Metadata for the hello screen and, later, About → Diagnostics.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Copyfie",
        version: env!("CARGO_PKG_VERSION"),
        core_version: copyfie_core::VERSION,
        identifier: "com.copyfie.app",
    }
}
