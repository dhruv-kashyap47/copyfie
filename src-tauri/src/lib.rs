//! Tauri shell for Copyfie.
//!
//! This crate wires the platform layer, the store, the search layer, and the
//! two webviews together. Business logic lives in `copyfie-core`; OS-specific
//! code lives under `platform/`. Keep this crate thin.

mod commands;
mod state;
mod store;
mod thumbs;

use tauri::Manager;

use state::AppState;
use store::StoreHandle;

pub fn run() {
    let builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::history::store_stats,
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(data_dir.join(thumbs::THUMBS_DIR))?;

            let database = copyfie_core::Database::open(data_dir.join("clipboard.db"))?;
            app.manage(AppState::new(StoreHandle::spawn(database)));
            Ok(())
        })
        // Content-addressed thumbnails: copyfie://<hash>.png (macOS/Linux) and
        // http://copyfie.localhost/<hash>.png (Windows).
        .register_uri_scheme_protocol("copyfie", |ctx, request| {
            match ctx.app_handle().path().app_data_dir() {
                Ok(dir) => thumbs::serve(&dir.join(thumbs::THUMBS_DIR), &request),
                Err(_) => thumbs::unavailable(),
            }
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running Copyfie");
}
