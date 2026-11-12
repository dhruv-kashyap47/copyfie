//! Platform contracts shared by the Windows, macOS, and Linux backends.
//!
//! Implementations live in `platform/{windows,macos,linux}`. This crate has no
//! OS APIs itself, so the capture pipeline can be tested with fakes.
//!
//! Two-phase reading is deliberate: [`ClipboardReader::probe`] exposes formats
//! only. Payload data is read **after** the privacy gate has accepted the
//! change. A concealed password-manager item is never read.

use copyfie_core::SourceApp;
use thiserror::Error;

pub mod focus;
pub mod source_app;

#[cfg(feature = "input")]
pub mod input;

#[cfg(feature = "system-clipboard")]
pub mod system;

pub use focus::{FocusBackend, FocusTarget};

/// Sends the paste chord (Ctrl+V / Cmd+V) to the foreground app.
pub trait PasteInput: Send + Sync {
    fn send_paste(&self) -> Result<(), PlatformError>;
}

/// Clipboard formats available for the current change, with no payload read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClipboardProbe {
    pub has_text: bool,
    /// Images land in M2-04.
    pub has_image: bool,
    /// File lists land in M2-05.
    pub has_files: bool,
    /// A password-manager/concealed marker is present.
    pub is_concealed: bool,
}

impl ClipboardProbe {
    pub const fn text() -> Self {
        Self {
            has_text: true,
            has_image: false,
            has_files: false,
            is_concealed: false,
        }
    }
}

/// Blocks until the OS clipboard changes and returns a monotonic sequence.
///
/// Windows uses `WM_CLIPBOARDUPDATE`, X11 uses XFixes selection events, and
/// macOS polls `NSPasteboard.changeCount` cheaply.
pub trait ClipboardWatcher: Send {
    fn wait_for_change(&mut self) -> Result<u64, PlatformError>;
}

/// Format probing and payload reading for the current sequence.
///
/// `sequence` guards against reading a newer clipboard state than the one that
/// was probed.
pub trait ClipboardReader: Send + Sync {
    fn probe(&self, sequence: u64) -> Result<ClipboardProbe, PlatformError>;

    /// Read UTF-8 text. `None` means the clipboard carries no text.
    fn read_text(&self, sequence: u64) -> Result<Option<String>, PlatformError>;
}

/// The app that owned the foreground window when the capture happened.
///
/// Linux Wayland cannot reliably provide this; implementations return `None`
/// and the UI shows "Unknown".
pub trait ForegroundApp: Send + Sync {
    fn current(&self) -> Option<SourceApp>;
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("clipboard is held by another application: {0}")]
    ClipboardBusy(String),

    #[error("clipboard access failed: {0}")]
    Clipboard(String),

    #[error("this platform does not support {0}")]
    Unsupported(&'static str),

    #[error("platform error: {0}")]
    Other(String),
}

// Blanket impls so tests and callers can share one backend behind an `Arc`.
impl<T: ClipboardWatcher + ?Sized> ClipboardWatcher for Box<T> {
    fn wait_for_change(&mut self) -> Result<u64, PlatformError> {
        (**self).wait_for_change()
    }
}

impl<T: ClipboardReader + ?Sized> ClipboardReader for std::sync::Arc<T> {
    fn probe(&self, sequence: u64) -> Result<ClipboardProbe, PlatformError> {
        (**self).probe(sequence)
    }

    fn read_text(&self, sequence: u64) -> Result<Option<String>, PlatformError> {
        (**self).read_text(sequence)
    }
}

impl<T: ForegroundApp + ?Sized> ForegroundApp for std::sync::Arc<T> {
    fn current(&self) -> Option<SourceApp> {
        (**self).current()
    }
}
