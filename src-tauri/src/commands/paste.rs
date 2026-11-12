//! Paste and copy commands.
//!
//! `paste_item` is the M1-08 orchestrator entry point. `copy_item` remains for
//! explicit copy-only actions (and will back the Wayland fallback).

use std::time::Instant;

use copyfie_core::ItemKind;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::capture::unix_ms;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasteReport {
    /// `pasted` when the chord was synthesized, `copied` on the fallback path.
    pub status: &'static str,
    pub reason: Option<String>,
}

#[tauri::command]
pub async fn copy_item(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let store = state.store.clone();
    let item = store
        .clone()
        .call(move |db| db.get(id))
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "item not found".to_string())?;

    require_text(&item)?;
    write_with_suppression(&item.text, &state.suppressor)?;

    let now = unix_ms();
    let _ = store.call(move |db| db.touch_used(id, now)).await;
    Ok(())
}

#[tauri::command]
pub async fn paste_item(
    id: i64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<PasteReport, String> {
    let store = state.store.clone();
    let suppressor = state.suppressor.clone();
    let paste = state.paste.clone();

    let item = store
        .clone()
        .call(move |db| db.get(id))
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "item not found".to_string())?;
    require_text(&item)?;

    let Some(target) = paste.take_target() else {
        // Shown via `--show` or a future tray action with no prior target.
        write_with_suppression(&item.text, &suppressor)?;
        touch(&store, id).await;
        return Ok(PasteReport {
            status: "copied",
            reason: Some("no target window was captured".to_string()),
        });
    };

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();

        // Hiding is dispatched to the event loop; wait until the OS reports the
        // palette hidden so focus restoration does not race the hide.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(400);
        while std::time::Instant::now() < deadline && window.is_visible().unwrap_or(false) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    let text = item.text.clone();
    let suppressor_for_task = suppressor.clone();
    let paste_for_task = paste.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        paste_for_task.perform(&target, &text, &suppressor_for_task)
    })
    .await
    .map_err(|error| error.to_string())?;

    match outcome {
        Ok(()) => {
            touch(&store, id).await;
            log::info!(target: "copyfie::paste", "pasted item id={id}");
            Ok(PasteReport {
                status: "pasted",
                reason: None,
            })
        }
        Err(reason) => {
            log::warn!(target: "copyfie::paste", "paste fell back to copy: {reason}");
            // Guarantee the content is on the clipboard for a manual paste.
            write_with_suppression(&item.text, &suppressor)?;
            let _ = crate::commands::window::show_palette(&app);
            Ok(PasteReport {
                status: "copied",
                reason: Some(reason),
            })
        }
    }
}

fn require_text(item: &copyfie_core::Item) -> Result<(), String> {
    if item.kind == ItemKind::Text {
        Ok(())
    } else {
        Err("only text items can be copied or pasted in this milestone".to_string())
    }
}

fn write_with_suppression(
    text: &str,
    suppressor: &crate::capture::Suppressor,
) -> Result<(), String> {
    suppressor.arm_text(text, Instant::now());
    if let Err(error) = crate::platform::write_text(text) {
        suppressor.clear();
        return Err(error.to_string());
    }
    Ok(())
}

async fn touch(store: &crate::store::StoreHandle, id: i64) {
    let now = unix_ms();
    let _ = store.call(move |db| db.touch_used(id, now)).await;
}
