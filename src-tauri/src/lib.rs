//! Tauri shell for Copyfie.
//!
//! This crate wires the platform layer, the store, the search layer, and the
//! two webviews together. Business logic lives in `copyfie-core`; OS-specific
//! code lives under `platform/`. Keep this crate thin.

mod commands;
mod state;
mod store;
mod thumbs;

pub mod capture;

use std::sync::{Arc, RwLock};

use tauri::Manager;

use capture::privacy::CaptureConfig;
use capture::suppress::Suppressor;
use state::AppState;
use store::StoreHandle;

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::history::store_stats,
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(data_dir.join(thumbs::THUMBS_DIR))?;

            let database = copyfie_core::Database::open(data_dir.join("clipboard.db"))?;
            let store = StoreHandle::spawn(database);

            // Capture shares its config and suppression token with the rest of
            // the app. They move into AppState when settings/paste land.
            let config = Arc::new(RwLock::new(CaptureConfig::default()));
            let suppressor = Arc::new(Suppressor::new());
            capture::spawn_capture(app.handle().clone(), store.clone(), config, suppressor);

            app.manage(AppState::new(store));
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
