//! Tauri shell for Copyfie.
//!
//! This crate wires the platform layer, the store, the search layer, and the
//! two webviews together. Business logic lives in `copyfie-core`; OS-specific
//! code lives under `platform/`. Keep this crate thin.

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    core_version: &'static str,
    identifier: &'static str,
}

/// Metadata for the hello screen and, later, About → Diagnostics.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "Copyfie",
        version: env!("CARGO_PKG_VERSION"),
        core_version: copyfie_core::VERSION,
        identifier: "com.copyfie.app",
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("error while running Copyfie");
}
