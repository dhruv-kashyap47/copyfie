//! System tray: show palette, pause capture, launch at login, quit.
//!
//! Icon and menu are Rust-side; the webview has no tray/menu permissions.

use tauri::menu::{CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::state::AppState;

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let paused = read_paused(app);
    let login_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    let show = MenuItemBuilder::with_id("show", "Show palette").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings…").build(app)?;
    let pause = CheckMenuItemBuilder::with_id("pause", "Pause capture")
        .checked(paused)
        .build(app)?;
    let login = CheckMenuItemBuilder::with_id("autostart", "Start at login")
        .checked(login_enabled)
        .build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Copyfie").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&settings)
        .item(&pause)
        .item(&login)
        .separator()
        .item(&quit)
        .build()?;

    let pause_item = pause.clone();
    let login_item = login.clone();

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Copyfie")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                let _ = crate::commands::window::show_palette(app);
            }
            "settings" => {
                // Window creation must not run on the main thread.
                let app = app.clone();
                std::thread::spawn(move || {
                    let _ = crate::commands::window::show_settings(&app);
                });
            }
            "pause" => {
                let paused = toggle_pause(app);
                let _ = pause_item.set_checked(paused);
                log::info!(target: "copyfie::tray", "capture paused={paused}");
            }
            "autostart" => {
                toggle_autostart(app, &login_item);
            }
            "quit" => {
                app.exit(0);
            }
            other => {
                log::debug!(target: "copyfie::tray", "unhandled tray item: {other}");
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = crate::commands::window::show_palette(tray.app_handle());
            }
        });

    #[cfg(target_os = "macos")]
    {
        // Derived monochrome silhouette of the supplied app icon; see
        // assets/logo/README.md. Kept as a template so macOS can tint it for
        // light and dark menu bars.
        match tauri::image::Image::from_bytes(include_bytes!("../icons/tray-template.png")) {
            Ok(icon) => {
                builder = builder.icon(icon).icon_as_template(true);
            }
            Err(error) => {
                log::warn!(target: "copyfie::tray", "could not load the template tray icon: {error}");
                if let Some(icon) = app.default_window_icon() {
                    builder = builder.icon(icon.clone());
                }
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn read_paused(app: &AppHandle) -> bool {
    let config = app.state::<AppState>().capture_config.clone();
    let guard = config.read().unwrap_or_else(|poison| poison.into_inner());
    guard.paused
}

fn toggle_pause(app: &AppHandle) -> bool {
    let config = app.state::<AppState>().capture_config.clone();
    let mut guard = config.write().unwrap_or_else(|poison| poison.into_inner());
    guard.paused = !guard.paused;
    guard.paused
}

fn toggle_autostart(app: &AppHandle, item: &CheckMenuItem<tauri::Wry>) {
    let manager = app.autolaunch();
    let enabled = manager.is_enabled().unwrap_or(false);

    let result = if enabled {
        manager.disable()
    } else {
        manager.enable()
    };

    match result {
        Ok(()) => {
            let next = !enabled;
            let _ = item.set_checked(next);
            log::info!(target: "copyfie::tray", "launch at login={next}");
        }
        Err(error) => {
            log::warn!(target: "copyfie::tray", "could not change autostart: {error}");
        }
    }
}
