//! Linux backend (v1.0 is X11-only).
//!
//! - Clipboard watch/read: shared `clipboard-rs` adapter (XFixes underneath).
//! - Foreground app: `_NET_ACTIVE_WINDOW` → `_NET_WM_PID` → `/proc/<pid>/comm`,
//!   with `WM_CLASS` as the stable fallback.
//! - Concealed content: the KDE password-manager hint.
//!
//! Wayland is detected at startup and kept in copy-only mode (M2-12). This
//! crate compiles on other hosts so workspace checks stay platform-agnostic;
//! all functions are inert without a display.

pub mod concealed;
pub mod foreground;

use copyfie_platform::system::{SystemClipboard, SystemWatcher};
use copyfie_platform::PlatformError;

pub use foreground::X11Foreground;

pub type Clipboard = SystemClipboard;
pub type Watcher = SystemWatcher;

pub fn new_clipboard() -> Result<Clipboard, PlatformError> {
    SystemClipboard::new(Box::new(concealed::LinuxConcealed))
}

pub fn new_watcher() -> Result<Watcher, PlatformError> {
    SystemWatcher::start()
}

pub fn new_foreground() -> X11Foreground {
    X11Foreground
}

/// True when the session is Wayland rather than X11.
pub fn is_wayland_session() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|value| value.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false)
}
