//! Tauri shell for Copyfie.
//!
//! This crate wires the platform layer, the store, the search layer, and the
//! two webviews together. Business logic lives in `copyfie-core`; OS-specific
//! code lives under `platform/`. Keep this crate thin.

mod commands;
mod hotkey;
mod paste;
mod platform;
mod settings;
mod state;
mod store;
mod thumbs;
mod tray;

pub mod capture;

use std::sync::{Arc, RwLock};

use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

use capture::privacy::CaptureConfig;
use capture::suppress::Suppressor;
use hotkey::HotkeyManager;
use paste::PasteService;
use settings::SettingsStore;
use state::AppState;
use store::StoreHandle;

pub fn run() {
    let builder = tauri::Builder::default()
        // Must be the first plugin: a second process exits here and forwards to
        // the running instance instead of starting a duplicate.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Err(error) = commands::window::show_palette(app) {
                log::warn!(target: "copyfie::window", "second instance could not show the palette: {error}");
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = commands::window::show_palette(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::history::store_stats,
            commands::history::history_list,
            commands::history::history_search,
            commands::history::history_pin,
            commands::history::history_delete,
            commands::history::history_clear,
            commands::paste::copy_item,
            commands::paste::paste_item,
            commands::settings::hotkey_status,
            commands::settings::set_hotkey,
            commands::settings::settings_get,
            commands::settings::settings_save,
            commands::window::hide_palette,
            commands::window::palette_show,
            commands::window::open_settings,
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(data_dir.join(thumbs::THUMBS_DIR))?;

            let settings_store = Arc::new(SettingsStore::new(&data_dir));
            let settings = settings_store.load();

            let database = copyfie_core::Database::open(data_dir.join("clipboard.db"))?;
            let store = StoreHandle::spawn(database);

            let capture_config = CaptureConfig {
                retention: copyfie_core::Retention::from_days(settings.retention_days),
                paused: settings.capture_paused,
                ..CaptureConfig::default()
            };
            let config = Arc::new(RwLock::new(capture_config));
            let suppressor = Arc::new(Suppressor::new());
            capture::spawn_capture(
                app.handle().clone(),
                store.clone(),
                config.clone(),
                suppressor.clone(),
            );

            let hotkey = Arc::new(HotkeyManager::new(settings.hotkey.clone()));

            app.manage(AppState::new(
                store,
                suppressor,
                Arc::new(PasteService::new()),
                config,
                hotkey.clone(),
                settings_store,
            ));

            tray::install(app.handle())?;

            let registered = match hotkey.try_register(app.handle(), &settings.hotkey) {
                Ok(()) => {
                    log::info!(target: "copyfie::hotkey", "global shortcut registered");
                    true
                }
                Err(error) => {
                    log::warn!(
                        target: "copyfie::hotkey",
                        "saved shortcut is unavailable; asking the user to rebind: {error}"
                    );
                    false
                }
            };
            hotkey.mark_initialized();

            if !registered {
                // Blocking setup step: the palette shows the recorder overlay.
                let _ = commands::window::show_palette(app.handle());
            }

            if std::env::args().any(|argument| argument == "--show") {
                if let Err(error) = commands::window::show_palette(app.handle()) {
                    log::error!(target: "copyfie::window", "could not show the palette: {error}");
                }
            }

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
