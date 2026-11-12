//! Paste and copy commands.
//!
//! M1-06 ships the copy-only path: the selected item is written back to the
//! clipboard with echo suppression armed first. The full orchestrator (hide
//! palette, restore focus, synthesize Ctrl/Cmd+V) lands in M1-08 and reuses
//! `copy_to_clipboard`.

use std::time::Instant;

use copyfie_core::ItemKind;
use tauri::State;

use crate::capture::unix_ms;
use crate::state::AppState;

#[tauri::command]
pub async fn copy_item(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let store = state.store.clone();
    let item = store
        .call(move |db| db.get(id))
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "item not found".to_string())?;

    if item.kind != ItemKind::Text {
        return Err("only text items are supported in this milestone".to_string());
    }

    // Arm before the write: the OS notification that follows must be ignored.
    state.suppressor.arm_text(&item.text, Instant::now());

    if let Err(error) = crate::platform::write_text(&item.text) {
        state.suppressor.clear();
        return Err(error.to_string());
    }

    let now = unix_ms();
    let _ = store.call(move |db| db.touch_used(id, now)).await;
    Ok(())
}
