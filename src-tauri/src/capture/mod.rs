//! Capture pipeline.
//!
//! Flow (PLAN §3):
//!
//! ```text
//! OS change ──► probe formats ──► privacy gate ──► read payload
//!                                      │
//!                           concealed / blocked / paused
//!                                      │
//!                                      └──► drop (nothing read, nothing stored)
//!                                                   │ allowed
//!                                                   ▼
//!                                     canonicalize ─► hash ─► echo suppression
//!                                                   ▼
//!                                     store actor upsert (dedupe)
//!                                                   ▼
//!                                     coalesced history:changed
//! ```
//!
//! Payloads are never read before the gate. A failed write never arms echo
//! suppression, so the same content can still be captured later.

pub mod events;
pub mod pipeline;
pub mod privacy;
pub mod source_app;
pub mod suppress;

use thiserror::Error;

pub use events::{tauri_history_events, EventCoalescer, HistoryEvents};
pub use pipeline::{CaptureOutcome, CapturePipeline, Clock};
pub use privacy::{
    matches_blocklist_entry, BlocklistEntry, CaptureConfig, GateDecision, Platform, PrivacyGate,
    SkipReason,
};
pub use source_app::{basename, display_name};
pub use suppress::Suppressor;

/// Current wall-clock time in Unix milliseconds.
pub fn unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

/// Errors surfaced by the capture pipeline.
#[derive(Debug, Error)]
pub enum CaptureError {
    #[error(transparent)]
    Platform(#[from] copyfie_platform::PlatformError),

    #[error(transparent)]
    Store(#[from] crate::store::ActorError),
}
