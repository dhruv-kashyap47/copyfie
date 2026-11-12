//! Coalesced `history:changed` notifications.
//!
//! Capture bursts (a multi-item copy, a paste echo, cleanup) must not wake the
//! webview per row. One emit per window is enough: the palette refetches from
//! the summary cache and ignores stale revisions.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Default debounce window: long enough to merge a burst, short enough that the
/// palette updates while the user is still looking at it.
pub const DEFAULT_WINDOW: Duration = Duration::from_millis(80);

/// Event name consumed by the palette.
pub const HISTORY_CHANGED: &str = "history:changed";

/// Anything that wants to know the store changed.
pub trait HistoryEvents: Send + Sync {
    fn changed(&self);
}

impl<T: HistoryEvents + ?Sized> HistoryEvents for Arc<T> {
    fn changed(&self) {
        (**self).changed();
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryChanged {
    pub revision: u64,
}

/// At-most-one-emit-per-window notifier with a monotonic revision.
pub struct EventCoalescer {
    tx: Sender<()>,
}

impl EventCoalescer {
    pub fn new<F>(window: Duration, emit: F) -> Self
    where
        F: Fn(u64) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<()>();
        let revision = AtomicU64::new(0);

        thread::Builder::new()
            .name("copyfie-events".to_string())
            .spawn(move || {
                while rx.recv().is_ok() {
                    // Merge everything that lands inside the window, then emit
                    // exactly once. Sustained traffic still emits per window.
                    thread::sleep(window);
                    while rx.try_recv().is_ok() {}

                    let next = revision.fetch_add(1, Ordering::Relaxed) + 1;
                    emit(next);
                }
            })
            .expect("failed to spawn the event coalescer thread");

        Self { tx }
    }

    /// Request a notification. Cheap: a channel send with no payload.
    pub fn notify(&self) {
        let _ = self.tx.send(());
    }
}

impl HistoryEvents for EventCoalescer {
    fn changed(&self) {
        self.notify();
    }
}

/// Wire the coalescer to the Tauri event bus.
pub fn tauri_history_events(app: AppHandle) -> EventCoalescer {
    EventCoalescer::new(DEFAULT_WINDOW, move |revision| {
        let _ = app.emit(HISTORY_CHANGED, HistoryChanged { revision });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn a_burst_emits_once() {
        let (tx, rx) = mpsc::channel();
        let coalescer = EventCoalescer::new(Duration::from_millis(150), move |revision| {
            let _ = tx.send(revision);
        });

        for _ in 0..25 {
            coalescer.changed();
        }

        let first = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("an event is emitted");
        assert_eq!(first, 1);

        assert!(
            rx.recv_timeout(Duration::from_millis(250)).is_err(),
            "a burst must not emit a second event"
        );
    }

    #[test]
    fn separate_bursts_advance_the_revision() {
        let (tx, rx) = mpsc::channel();
        let coalescer = EventCoalescer::new(Duration::from_millis(80), move |revision| {
            let _ = tx.send(revision);
        });

        coalescer.changed();
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).expect("first"), 1);

        coalescer.changed();
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).expect("second"), 2);
    }

    #[test]
    fn notification_is_cheap_under_a_long_burst() {
        let (tx, rx) = mpsc::channel();
        let coalescer = EventCoalescer::new(Duration::from_millis(100), move |revision| {
            let _ = tx.send(revision);
        });

        let start = Instant::now();
        for _ in 0..10_000 {
            coalescer.notify();
        }
        assert!(
            start.elapsed() < Duration::from_millis(500),
            "notify must not block on the emit path"
        );

        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).expect("event"), 1);
    }
}
