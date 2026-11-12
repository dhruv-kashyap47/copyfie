//! History commands.
//!
//! Thin wrappers over the store actor. Search ranking lives in
//! `copyfie-core`; this layer only validates IPC input and maps errors.

use copyfie_core::{ItemKind, ItemSummary, ListQuery, Retention};
use serde::Serialize;
use tauri::State;

use crate::capture::unix_ms;
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreStats {
    pub total: i64,
    pub text: i64,
    pub images: i64,
    pub files: i64,
}

#[tauri::command]
pub async fn store_stats(state: State<'_, AppState>) -> Result<StoreStats, String> {
    let store = state.store.clone();

    store
        .call(|db| {
            Ok(StoreStats {
                total: db.count(None)?,
                text: db.count(Some(ItemKind::Text))?,
                images: db.count(Some(ItemKind::Image))?,
                files: db.count(Some(ItemKind::Files))?,
            })
        })
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn history_list(
    limit: Option<u32>,
    offset: Option<u32>,
    kind: Option<String>,
    pinned_only: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<ItemSummary>, String> {
    let kind = parse_kind(kind.as_deref())?;
    let query = ListQuery {
        limit: limit.unwrap_or(100).clamp(1, 100),
        offset: offset.unwrap_or(0),
        kind,
        pinned_only: pinned_only.unwrap_or(false),
    };

    let store = state.store.clone();
    store
        .call(move |db| db.list(&query))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn history_search(
    query: String,
    kind: Option<String>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<ItemSummary>, String> {
    let kind = parse_kind(kind.as_deref())?;
    let limit = limit.unwrap_or(copyfie_core::search::MAX_RESULTS);

    let store = state.store.clone();
    store
        .call(move |db| db.search(&query, kind, limit))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn history_pin(
    id: i64,
    pinned: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let store = state.store.clone();
    let now = unix_ms();
    store
        .call(move |db| db.set_pinned(id, pinned, now, Retention::default()))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn history_delete(id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let store = state.store.clone();
    store
        .call(move |db| db.delete(id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn history_clear(keep_pinned: bool, state: State<'_, AppState>) -> Result<usize, String> {
    let store = state.store.clone();
    store
        .call(move |db| db.clear(keep_pinned))
        .await
        .map_err(|error| error.to_string())
}

fn parse_kind(kind: Option<&str>) -> Result<Option<ItemKind>, String> {
    match kind.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("all") => Ok(None),
        Some("text") => Ok(Some(ItemKind::Text)),
        Some("image") => Ok(Some(ItemKind::Image)),
        Some("files") => Ok(Some(ItemKind::Files)),
        Some(other) => Err(format!("unknown item kind: {other}")),
    }
}
