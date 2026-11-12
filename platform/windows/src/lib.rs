//! Windows backend.
//!
//! - Clipboard watch/read: shared `clipboard-rs` adapter (`WM_CLIPBOARDUPDATE`
//!   underneath).
//! - Foreground app: `GetForegroundWindow` → PID → `QueryFullProcessImageNameW`.
//! - Concealed content: clipboard format markers, including the DWORD-valued
//!   Office markers, which are only concealed when set to `0`.
#![cfg(target_os = "windows")]

pub mod concealed;
pub mod focus;
pub mod foreground;

use copyfie_platform::system::{SystemClipboard, SystemWatcher};
use copyfie_platform::{FocusBackend, PasteInput, PlatformError};

pub use focus::WindowsFocus;
pub use foreground::WindowsForeground;

pub type Clipboard = SystemClipboard;
pub type Watcher = SystemWatcher;
pub type Paste = copyfie_platform::input::SystemPasteInput;

pub fn new_clipboard() -> Result<Clipboard, PlatformError> {
    SystemClipboard::new(Box::new(concealed::WindowsConcealed))
}

pub fn new_watcher() -> Result<Watcher, PlatformError> {
    SystemWatcher::start()
}

pub fn new_foreground() -> WindowsForeground {
    WindowsForeground
}

pub fn new_focus() -> Box<dyn FocusBackend> {
    Box::new(WindowsFocus)
}

pub fn new_paste() -> Box<dyn PasteInput> {
    Box::new(copyfie_platform::input::SystemPasteInput)
}

pub fn write_text(text: &str) -> Result<(), PlatformError> {
    copyfie_platform::system::write_text(text)
}
