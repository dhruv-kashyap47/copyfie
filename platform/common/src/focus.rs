//! Focus capture/restore for the paste orchestrator.
//!
//! The target is captured **before** the palette takes focus and restored by
//! handle/pid afterwards. The orchestrator polls `is_active` instead of
//! sleeping a fixed amount, so paste happens only into the intended window.

/// Platform-specific identity of the window that should receive the paste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusTarget {
    Windows { hwnd: isize, pid: u32 },
    MacOs { pid: i32 },
    X11 { window: u32 },
}

pub trait FocusBackend: Send + Sync {
    /// Foreground window/app right now.
    fn capture(&self) -> Option<FocusTarget>;

    /// Ask the OS to bring the target back to the foreground.
    fn restore(&self, target: &FocusTarget) -> Result<(), super::PlatformError>;

    /// True when the target is the active window/app.
    fn is_active(&self, target: &FocusTarget) -> bool;
}
