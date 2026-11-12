//! Windows concealed/password-manager clipboard detection.
//!
//! Markers fall into two classes:
//! - **Presence** means "do not store" (KeePassXC, Bitwarden, Windows
//!   exclusion markers).
//! - **Value** means "do not store only when set to 0" (Office sets
//!   `CanIncludeInClipboardHistory=1` for ordinary copies).
//!
//! If a value marker is present but unreadable (clipboard held by another
//! process), we fail closed and treat the item as concealed.

use std::os::windows::ffi::OsStrExt;
use std::time::Duration;

use copyfie_platform::system::ConcealedProbe;
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, OpenClipboard, RegisterClipboardFormatW,
};
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};

/// Windows, KeePassXC, and Bitwarden markers.
const PRESENCE_MARKERS: &[&str] = &[
    "ExcludeClipboardContentFromMonitorProcessing",
    "ClipboardViewerIgnore",
    "KeePassClipFormat",
    "com.bitwarden.concealed",
];

/// Office/Windows markers whose DWORD value decides concealment.
const VALUE_MARKERS: &[&str] = &["CanIncludeInClipboardHistory", "CanUploadToCloudClipboard"];

pub struct WindowsConcealed;

impl ConcealedProbe for WindowsConcealed {
    fn is_concealed(&self, formats: &[String]) -> bool {
        if formats.iter().any(|format| {
            PRESENCE_MARKERS
                .iter()
                .any(|marker| format.eq_ignore_ascii_case(marker))
        }) {
            return true;
        }

        for marker in VALUE_MARKERS {
            let present = formats
                .iter()
                .any(|format| format.eq_ignore_ascii_case(marker));
            if !present {
                continue;
            }

            match read_dword_format(marker) {
                Some(0) | None => return true,
                Some(_) => {}
            }
        }

        false
    }
}

/// Read a DWORD-valued clipboard format with the retry policy from PLAN §6.
fn read_dword_format(name: &str) -> Option<u32> {
    for attempt in 0..3 {
        if let Some(value) = read_dword_once(name) {
            return Some(value);
        }
        if attempt < 2 {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    None
}

fn read_dword_once(name: &str) -> Option<u32> {
    let wide: Vec<u16> = std::ffi::OsStr::new(name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: all handles are checked; the clipboard is closed on every path
    // that opened it. `GlobalLock` is paired with `GlobalUnlock`.
    unsafe {
        let format = RegisterClipboardFormatW(wide.as_ptr());
        if format == 0 {
            return None;
        }

        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return None;
        }

        let handle = GetClipboardData(format);
        let value = if handle.is_null() {
            None
        } else {
            let pointer = GlobalLock(handle);
            if pointer.is_null() {
                None
            } else {
                let value = std::ptr::read_unaligned(pointer.cast::<u32>());
                GlobalUnlock(handle);
                Some(value)
            }
        };

        CloseClipboard();
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formats(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn detects_presence_markers_case_insensitively() {
        let probe = WindowsConcealed;
        assert!(probe.is_concealed(&formats(&["KEEpassClipFormat"])));
        assert!(probe.is_concealed(&formats(&["CF_UNICODETEXT", "com.bitwarden.concealed"])));
        assert!(probe.is_concealed(&formats(&["ExcludeClipboardContentFromMonitorProcessing"])));
        assert!(probe.is_concealed(&formats(&["ClipboardViewerIgnore"])));
    }

    #[test]
    fn ordinary_text_is_not_concealed() {
        let probe = WindowsConcealed;
        assert!(!probe.is_concealed(&formats(&["CF_UNICODETEXT", "CF_TEXT"])));
        assert!(!probe.is_concealed(&formats(&[])));
    }

    #[test]
    fn value_markers_only_matter_when_present() {
        // Absent value markers never open the clipboard in tests, so the result
        // is false even though the registry format does not exist yet.
        let probe = WindowsConcealed;
        assert!(!probe.is_concealed(&formats(&["CF_UNICODETEXT"])));
    }
}
