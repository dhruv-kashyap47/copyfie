//! SQLite persistence.
//!
//! One `Database` owns one `Connection`. The app wraps it in a store actor
//! (single writer thread) — this module stays synchronous and thread-agnostic
//! so it can be tested headlessly.

mod db;
mod items;
mod migrate;
mod retention;

use thiserror::Error;

pub use db::Database;
pub(crate) use items::summary_from_row;

/// Outcome of an upsert: the row identity plus whether it is new content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsertOutcome {
    pub id: i64,
    /// `true` when this content had never been seen before.
    pub is_new: bool,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "database schema v{found} is newer than this build supports (v{supported}); \
         update Copyfie instead of downgrading"
    )]
    FutureSchema { found: i64, supported: i64 },

    #[error("migration {name} failed: {source}")]
    Migration {
        name: &'static str,
        #[source]
        source: rusqlite::Error,
    },
}

pub type Result<T> = std::result::Result<T, StoreError>;
