//! Windows backend.
//!
//! - Clipboard watch/read: shared `clipboard-rs` adapter (`WM_CLIPBOARDUPDATE`
//!   underneath).
//! - Foreground app: `GetForegroundWindow` → PID → `QueryFullProcessImageNameW`.
//! - Concealed content: clipboard format markers, including the DWORD-valued
//!   Office markers, which are only concealed when set to `0`.
#![cfg(target_os = "windows")]

pub mod concealed;
pub mod foreground;

use copyfie_platform::system::{SystemClipboard, SystemWatcher};
use copyfie_platform::PlatformError;

pub use foreground::WindowsForeground;

pub type Clipboard = SystemClipboard;
pub type Watcher = SystemWatcher;

pub fn new_clipboard() -> Result<Clipboard, PlatformError> {
    SystemClipboard::new(Box::new(concealed::WindowsConcealed))
}

pub fn new_watcher() -> Result<Watcher, PlatformError> {
    SystemWatcher::start()
}

pub fn new_foreground() -> WindowsForeground {
    WindowsForeground
}
