//! macOS backend.
//!
//! - Clipboard watch/read: shared `clipboard-rs` adapter (`NSPasteboard`
//!   `changeCount` polling internally).
//! - Foreground app: `NSWorkspace.frontmostApplication` (bundle id + localized
//!   name). The window title needs Accessibility and is not read here.
//! - Concealed content: `org.nspasteboard.*` and password-manager marker types.
#![cfg(target_os = "macos")]

pub mod concealed;
pub mod foreground;

use copyfie_platform::system::{SystemClipboard, SystemWatcher};
use copyfie_platform::PlatformError;

pub use foreground::MacForeground;

pub type Clipboard = SystemClipboard;
pub type Watcher = SystemWatcher;

pub fn new_clipboard() -> Result<Clipboard, PlatformError> {
    SystemClipboard::new(Box::new(concealed::MacConcealed))
}

pub fn new_watcher() -> Result<Watcher, PlatformError> {
    SystemWatcher::start()
}

pub fn new_foreground() -> MacForeground {
    MacForeground
}

pub fn write_text(text: &str) -> Result<(), PlatformError> {
    copyfie_platform::system::write_text(text)
}
