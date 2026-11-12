//! Echo suppression.
//!
//! Writing an item back to the clipboard (paste, copy, plain-text paste) must
//! not create a new history row or bump the existing one. The paste
//! orchestrator arms a token before the write; the capture pipeline consumes it
//! once when the OS reports the same content.
//!
//! Properties:
//! - **Content-addressed:** a different copy during the window is unaffected.
//! - **One-shot:** consuming clears the token, so a deliberate re-copy of the
//!   same text two seconds later is captured.
//! - **TTL:** a stale token can never suppress a later user action.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use copyfie_core::hash;

/// How long an armed token stays valid.
pub const DEFAULT_TTL: Duration = Duration::from_secs(2);

#[derive(Debug)]
struct Armed {
    hash: [u8; 32],
    expires_at: Instant,
}

/// Thread-safe one-shot suppression token.
#[derive(Debug)]
pub struct Suppressor {
    ttl: Duration,
    armed: Mutex<Option<Armed>>,
}

impl Suppressor {
    pub fn new() -> Self {
        Self::with_ttl(DEFAULT_TTL)
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            ttl,
            armed: Mutex::new(None),
        }
    }

    /// Arm a token for the content we are about to write.
    pub fn arm(&self, content_hash: [u8; 32], now: Instant) {
        let mut armed = self.lock();
        *armed = Some(Armed {
            hash: content_hash,
            expires_at: now + self.ttl,
        });
    }

    /// Convenience wrapper for text payloads.
    pub fn arm_text(&self, text: &str, now: Instant) {
        self.arm(hash::hash_text(text), now);
    }

    /// Consume the token when it matches. Expired tokens never match.
    pub fn consume_if_suppressed(&self, content_hash: &[u8; 32], now: Instant) -> bool {
        let mut armed = self.lock();

        match armed.as_ref() {
            Some(token) if token.hash == *content_hash && now < token.expires_at => {
                *armed = None;
                true
            }
            Some(token) if now >= token.expires_at => {
                *armed = None;
                false
            }
            _ => false,
        }
    }

    /// Drop any pending token (e.g. the write failed before the OS noticed).
    pub fn clear(&self) {
        *self.lock() = None;
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Armed>> {
        self.armed
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

impl Default for Suppressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_of(text: &str) -> [u8; 32] {
        hash::hash_text(text)
    }

    #[test]
    fn suppresses_the_armed_content_once() {
        let suppressor = Suppressor::new();
        let now = Instant::now();
        let token = hash_of("pasted text");

        suppressor.arm(token, now);

        assert!(suppressor.consume_if_suppressed(&token, now));
        assert!(
            !suppressor.consume_if_suppressed(&token, now),
            "tokens are one-shot"
        );
    }

    #[test]
    fn ignores_different_content() {
        let suppressor = Suppressor::new();
        let now = Instant::now();
        suppressor.arm(hash_of("pasted text"), now);

        assert!(!suppressor.consume_if_suppressed(&hash_of("user copy"), now));
        // The original token is still armed for the echo that follows.
        assert!(suppressor.consume_if_suppressed(&hash_of("pasted text"), now));
    }

    #[test]
    fn expires_after_the_ttl() {
        let suppressor = Suppressor::with_ttl(Duration::from_millis(50));
        let now = Instant::now();
        let token = hash_of("stale");

        suppressor.arm(token, now);
        assert!(!suppressor.consume_if_suppressed(&token, now + Duration::from_millis(51)));
        assert!(
            !suppressor.consume_if_suppressed(&token, now + Duration::from_millis(60)),
            "an expired token is cleared, not resurrected"
        );
    }

    #[test]
    fn rearming_replaces_the_previous_token() {
        let suppressor = Suppressor::new();
        let now = Instant::now();

        suppressor.arm(hash_of("first"), now);
        suppressor.arm(hash_of("second"), now);

        assert!(!suppressor.consume_if_suppressed(&hash_of("first"), now));
        assert!(suppressor.consume_if_suppressed(&hash_of("second"), now));
    }

    #[test]
    fn clear_removes_the_token() {
        let suppressor = Suppressor::new();
        let now = Instant::now();
        suppressor.arm(hash_of("x"), now);
        suppressor.clear();
        assert!(!suppressor.consume_if_suppressed(&hash_of("x"), now));
    }

    #[test]
    fn canonicalization_matches_captured_text() {
        let suppressor = Suppressor::new();
        let now = Instant::now();
        suppressor.arm_text("a\r\nb", now);
        assert!(suppressor.consume_if_suppressed(&hash::hash_text("a\nb"), now));
    }
}
