//! Capture and restore the foreground window.
//!
//! Restore uses the input-queue attachment trick so Windows lets a background
//! process raise the target, then the orchestrator polls `is_active` before
//! writing or pasting.

use copyfie_platform::{FocusBackend, FocusTarget, PlatformError};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, SetForegroundWindow,
    ShowWindow, SW_RESTORE,
};

pub struct WindowsFocus;

impl FocusBackend for WindowsFocus {
    fn capture(&self) -> Option<FocusTarget> {
        // SAFETY: reads the current foreground window; no handles are retained.
        unsafe {
            let window = GetForegroundWindow();
            if window.is_null() {
                return None;
            }

            let mut pid = 0_u32;
            GetWindowThreadProcessId(window, &mut pid);

            Some(FocusTarget::Windows {
                hwnd: window as isize,
                pid,
            })
        }
    }

    fn restore(&self, target: &FocusTarget) -> Result<(), PlatformError> {
        let FocusTarget::Windows { hwnd, .. } = *target else {
            return Err(PlatformError::Other(
                "focus target does not belong to Windows".to_string(),
            ));
        };

        let window = hwnd as HWND;

        // SAFETY: `window` comes from a prior `GetForegroundWindow` call in this
        // process. The input attachment is always detached before returning.
        unsafe {
            if IsIconic(window) != 0 {
                ShowWindow(window, SW_RESTORE);
            }

            let foreground = GetForegroundWindow();
            let foreground_thread = GetWindowThreadProcessId(foreground, std::ptr::null_mut());
            let current_thread = GetCurrentThreadId();

            let attached = foreground_thread != 0
                && foreground_thread != current_thread
                && AttachThreadInput(current_thread, foreground_thread, 1) != 0;

            BringWindowToTop(window);
            SetForegroundWindow(window);

            if attached {
                AttachThreadInput(current_thread, foreground_thread, 0);
            }
        }

        Ok(())
    }

    fn is_active(&self, target: &FocusTarget) -> bool {
        let FocusTarget::Windows { hwnd, .. } = *target else {
            return false;
        };

        // SAFETY: comparison of two window handles.
        unsafe { GetForegroundWindow() == hwnd as HWND }
    }
}
