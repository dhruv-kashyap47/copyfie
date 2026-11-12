//! Capture runner: one thread per process, platform-selected backend.
//!
//! The thread blocks on OS clipboard notifications and feeds them into the
//! pipeline. Errors are logged (never content) and do not kill the loop: a
//! transient clipboard lock must not stop capture until the next restart.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use copyfie_platform::ClipboardWatcher;
use tauri::AppHandle;

use crate::capture::events::tauri_history_events;
use crate::capture::pipeline::{CaptureOutcome, CapturePipeline};
use crate::capture::privacy::CaptureConfig;
use crate::capture::suppress::Suppressor;
use crate::store::StoreHandle;

#[cfg(target_os = "windows")]
use copyfie_platform_windows as platform;

#[cfg(target_os = "macos")]
use copyfie_platform_macos as platform;

#[cfg(target_os = "linux")]
use copyfie_platform_linux as platform;

/// Start clipboard capture on a dedicated thread.
pub fn spawn_capture(
    app: AppHandle,
    store: StoreHandle,
    config: Arc<RwLock<CaptureConfig>>,
    suppressor: Arc<Suppressor>,
) {
    std::thread::Builder::new()
        .name("copyfie-capture".to_string())
        .spawn(move || run(app, store, config, suppressor))
        .expect("failed to spawn the capture thread");
}

fn run(
    app: AppHandle,
    store: StoreHandle,
    config: Arc<RwLock<CaptureConfig>>,
    suppressor: Arc<Suppressor>,
) {
    #[cfg(target_os = "linux")]
    if platform::is_wayland_session() {
        // v1.0 is X11-only. Wayland capture is deliberately not attempted;
        // M2-12 surfaces the capability notice and copy-only path.
        log::warn!(
            target: "copyfie::capture",
            "Wayland session detected: background capture is X11-only in v1.0"
        );
        return;
    }

    let clipboard = match platform::new_clipboard() {
        Ok(clipboard) => clipboard,
        Err(error) => {
            log::error!(target: "copyfie::capture", "clipboard reader unavailable: {error}");
            return;
        }
    };

    let mut watcher = match platform::new_watcher() {
        Ok(watcher) => watcher,
        Err(error) => {
            log::error!(target: "copyfie::capture", "clipboard watcher unavailable: {error}");
            return;
        }
    };

    let events = tauri_history_events(app);
    let pipeline = CapturePipeline::new(
        clipboard,
        platform::new_foreground(),
        store,
        config,
        suppressor,
        events,
    );

    log::info!(target: "copyfie::capture", "clipboard capture started");

    loop {
        match watcher.wait_for_change() {
            Ok(sequence) => match pipeline.handle_change(sequence) {
                Ok(CaptureOutcome::Stored { id, is_new }) => {
                    log::info!(target: "copyfie::capture", "captured id={id} new={is_new}");
                }
                Ok(CaptureOutcome::Skipped(reason)) => {
                    log::debug!(target: "copyfie::capture", "skipped: {}", reason.as_str());
                }
                Err(error) => {
                    // A failed write is retryable: no suppression token was
                    // consumed and no history event was emitted.
                    log::warn!(target: "copyfie::capture", "capture failed: {error}");
                }
            },
            Err(error) => {
                log::warn!(target: "copyfie::capture", "clipboard watcher error: {error}");
                // Avoid a hot loop if the backend died.
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }
}
