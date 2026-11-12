//! Copyfie core.
//!
//! This crate owns models, canonicalization, hashing, the SQLite store, FTS5
//! search, retention, and the settings model.
//!
//! Hard rules (enforced by review and CI):
//! - No Tauri types.
//! - No OS/platform APIs. Everything here must be unit-testable headlessly.
//! - No clipboard reads. Payloads are handed in by the platform layer.

pub mod hash;
pub mod model;
pub mod retention;
pub mod search;
pub mod store;

pub use model::{Item, ItemKind, ItemSummary, ListQuery, NewItem, SourceApp};
pub use retention::Retention;
pub use store::{Database, InsertOutcome, StoreError};

/// Semantic version of the core crate, for diagnostics.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
