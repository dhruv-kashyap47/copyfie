//! Copyfie core.
//!
//! This crate owns models, canonicalization, hashing, the SQLite store, FTS5
//! search, retention, and the settings model.
//!
//! Hard rules (enforced by review and CI):
//! - No Tauri types.
//! - No OS/platform APIs. Everything here must be unit-testable headlessly.
//! - No clipboard reads. Payloads are handed in by the platform layer.
//!
//! Modules are added as the corresponding M1 tasks land (`model`, `hash`,
//! `store`, `search`, `settings`, `retention`).

/// Semantic version of the core crate, for diagnostics.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_reported() {
        assert!(!super::VERSION.is_empty());
    }
}
