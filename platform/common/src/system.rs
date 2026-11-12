//! Shared `clipboard-rs` adapter used by all desktop backends.
//!
//! The OS-specific crates supply only what differs: concealed-marker detection
//! and foreground-app identity. Watching and text reads are the same code path
//! on Windows, macOS, and X11, which keeps behaviour consistent and testable.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, MutexGuard};
use std::thread;

use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
    ContentFormat,
};

use crate::{ClipboardProbe, ClipboardReader, ClipboardWatcher as Watcher, PlatformError};

/// OS-specific concealed/password-manager detection over the format list.
pub trait ConcealedProbe: Send + Sync {
    fn is_concealed(&self, formats: &[String]) -> bool;
}

/// Text-only clipboard reader. Images and files are probed but not read until
/// M2-04/M2-05; a file selection is skipped rather than stored as path text.
pub struct SystemClipboard {
    inner: Mutex<ClipboardContext>,
    concealed: Box<dyn ConcealedProbe>,
}

impl SystemClipboard {
    pub fn new(concealed: Box<dyn ConcealedProbe>) -> Result<Self, PlatformError> {
        let context =
            ClipboardContext::new().map_err(|error| PlatformError::Clipboard(error.to_string()))?;
        Ok(Self {
            inner: Mutex::new(context),
            concealed,
        })
    }

    fn lock(&self) -> MutexGuard<'_, ClipboardContext> {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

impl ClipboardReader for SystemClipboard {
    fn probe(&self, _sequence: u64) -> Result<ClipboardProbe, PlatformError> {
        let context = self.lock();

        // Fail closed: if formats cannot be enumerated, the caller gets an
        // error and nothing is read, instead of guessing that it is safe.
        let formats = context
            .available_formats()
            .map_err(|error| PlatformError::Clipboard(error.to_string()))?;

        let is_concealed = self.concealed.is_concealed(&formats);
        let has_files = context.has(ContentFormat::Files);
        let has_text = !has_files && context.has(ContentFormat::Text);

        Ok(ClipboardProbe {
            has_text,
            has_image: false,
            has_files,
            is_concealed,
        })
    }

    fn read_text(&self, _sequence: u64) -> Result<Option<String>, PlatformError> {
        let context = self.lock();

        match context.get_text() {
            Ok(text) if !text.is_empty() => Ok(Some(text)),
            Ok(_) => Ok(None),
            Err(error) => Err(PlatformError::Clipboard(error.to_string())),
        }
    }
}

/// Event-driven (Windows/X11) or cheap-poll (macOS) watcher from `clipboard-rs`.
pub struct SystemWatcher {
    rx: Receiver<u64>,
}

impl SystemWatcher {
    /// Start the watcher thread and wait until the backend is confirmed up, so
    /// startup failures are reported instead of silently capturing nothing.
    pub fn start() -> Result<Self, PlatformError> {
        let (tx, rx) = mpsc::channel::<u64>();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<(), PlatformError>>(1);

        thread::Builder::new()
            .name("copyfie-clipboard".to_string())
            .spawn(move || {
                let mut watcher = match ClipboardWatcherContext::new() {
                    Ok(watcher) => watcher,
                    Err(error) => {
                        let _ = ready_tx.send(Err(PlatformError::Clipboard(error.to_string())));
                        return;
                    }
                };

                let _shutdown = watcher
                    .add_handler(ChangeHandler { tx, sequence: 0 })
                    .get_shutdown_channel();

                if ready_tx.send(Ok(())).is_err() {
                    return; // Caller gave up; nothing to watch for.
                }

                watcher.start_watch();
            })
            .map_err(|error| {
                PlatformError::Other(format!("failed to start clipboard thread: {error}"))
            })?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self { rx }),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(PlatformError::Other(
                "clipboard watcher failed to start".to_string(),
            )),
        }
    }
}

impl Watcher for SystemWatcher {
    fn wait_for_change(&mut self) -> Result<u64, PlatformError> {
        self.rx
            .recv()
            .map_err(|_| PlatformError::Other("clipboard watcher stopped".to_string()))
    }
}

struct ChangeHandler {
    tx: Sender<u64>,
    sequence: u64,
}

impl ClipboardHandler for ChangeHandler {
    fn on_clipboard_change(&mut self) {
        self.sequence = self.sequence.wrapping_add(1);
        // If the consumer is gone, the send fails quietly and the watcher keeps
        // running until the process exits.
        let _ = self.tx.send(self.sequence);
    }
}
