//! History/store commands.
//!
//! Listing, search, pin, delete, and clear land with the palette (M1-04…M1-06).
//! Until then this exposes the store health used by the scaffold screen and by
//! the M1-12 diagnostics.

use serde::Serialize;
use tauri::State;

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
                text: db.count(Some(copyfie_core::ItemKind::Text))?,
                images: db.count(Some(copyfie_core::ItemKind::Image))?,
                files: db.count(Some(copyfie_core::ItemKind::Files))?,
            })
        })
        .await
        .map_err(|error| error.to_string())
}
