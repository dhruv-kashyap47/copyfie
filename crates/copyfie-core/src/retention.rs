//! Retention policy.
//!
//! Expiry is stored per row (`expires_at`) so the palette and cleanup never
//! need to re-derive policy. Pinned items always have `expires_at = NULL`.
//!
//! The trial/license state never affects retention: a locked install keeps
//! expiring exactly as the user configured.

use serde::{Deserialize, Serialize};

/// One day in milliseconds.
pub const DAY_MS: i64 = 86_400_000;

/// User setting for how long unpinned items live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Retention {
    /// `0` is the internal sentinel for "Forever" in `settings.json`.
    Days(u32),
    Forever,
}

impl Default for Retention {
    fn default() -> Self {
        Self::Days(30)
    }
}

impl Retention {
    /// Interpret the settings-file representation: `0` means Forever.
    pub fn from_days(days: u32) -> Self {
        if days == 0 {
            Self::Forever
        } else {
            Self::Days(days.clamp(1, 365))
        }
    }

    /// `None` for Forever, otherwise the clamped day count.
    pub fn days(self) -> Option<u32> {
        match self {
            Self::Days(days) => Some(days.clamp(1, 365)),
            Self::Forever => None,
        }
    }
}

/// Absolute expiry timestamp for an item captured at `captured_at`.
pub fn expiry_for(captured_at: i64, retention: Retention) -> Option<i64> {
    retention
        .days()
        .map(|days| captured_at.saturating_add(i64::from(days) * DAY_MS))
}

/// Recompute expiry after unpinning.
pub fn expiry_after_unpin(captured_at: i64, retention: Retention) -> Option<i64> {
    expiry_for(captured_at, retention)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_thirty_days() {
        assert_eq!(Retention::default(), Retention::Days(30));
        assert_eq!(expiry_for(0, Retention::default()), Some(30 * DAY_MS));
    }

    #[test]
    fn zero_means_forever() {
        assert_eq!(Retention::from_days(0), Retention::Forever);
        assert_eq!(expiry_for(123, Retention::from_days(0)), None);
    }

    #[test]
    fn clamps_day_counts() {
        assert_eq!(Retention::from_days(9_999), Retention::Days(365));
        assert_eq!(Retention::from_days(7).days(), Some(7));
        assert_eq!(Retention::Forever.days(), None);
    }

    #[test]
    fn expiry_saturates_instead_of_overflowing() {
        assert_eq!(expiry_for(i64::MAX, Retention::Days(365)), Some(i64::MAX));
    }
}
