//! Foreground app identity on Windows.
//!
//! `GetForegroundWindow` → `GetWindowThreadProcessId` → `OpenProcess` →
//! `QueryFullProcessImageNameW`, plus the window title. No cache: this runs
//! once per accepted capture.

use copyfie_core::SourceApp;
use copyfie_platform::ForegroundApp;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

pub struct WindowsForeground;

impl ForegroundApp for WindowsForeground {
    fn current(&self) -> Option<SourceApp> {
        // SAFETY: every pointer is to a local buffer or a checked handle, and
        // every opened handle is closed before returning.
        unsafe { foreground() }
    }
}

unsafe fn foreground() -> Option<SourceApp> {
    let window = GetForegroundWindow();
    if window.is_null() {
        return None;
    }

    let mut pid = 0_u32;
    GetWindowThreadProcessId(window, &mut pid);

    let window_title = window_title(window);
    if pid == 0 {
        return Some(SourceApp {
            id: None,
            name: None,
            window_title,
        });
    }

    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if process.is_null() {
        return Some(SourceApp {
            id: None,
            name: None,
            window_title,
        });
    }

    let path = process_image_path(process);
    CloseHandle(process);

    let name = path.as_deref().and_then(friendly_name);
    Some(SourceApp {
        id: path,
        name,
        window_title,
    })
}

unsafe fn process_image_path(process: *mut core::ffi::c_void) -> Option<String> {
    let mut buffer = [0_u16; 1024];
    let mut size = buffer.len() as u32;

    let ok =
        QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buffer.as_mut_ptr(), &mut size);
    if ok == 0 || size == 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..size as usize]))
}

unsafe fn window_title(window: *mut core::ffi::c_void) -> Option<String> {
    let length = GetWindowTextLengthW(window);
    if length <= 0 {
        return None;
    }

    let mut buffer = vec![0_u16; length as usize + 1];
    let copied = GetWindowTextW(window, buffer.as_mut_ptr(), buffer.len() as i32);
    if copied <= 0 {
        return None;
    }

    let title = String::from_utf16_lossy(&buffer[..copied as usize]);
    let title = title.trim().to_string();
    (!title.is_empty()).then_some(title)
}

fn friendly_name(path: &str) -> Option<String> {
    let file = path.rsplit(['\\', '/']).next()?;
    let stem = file
        .strip_suffix(".exe")
        .or_else(|| file.strip_suffix(".EXE"))
        .unwrap_or(file);
    (!stem.is_empty()).then(|| stem.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_a_friendly_name_from_the_executable() {
        assert_eq!(
            friendly_name(r"C:\Program Files\Notepad\notepad.exe").as_deref(),
            Some("notepad")
        );
        assert_eq!(
            friendly_name("C:\\Apps\\KeePassXC.EXE").as_deref(),
            Some("KeePassXC")
        );
        assert_eq!(friendly_name("C:\\Apps\\tool").as_deref(), Some("tool"));
        assert_eq!(friendly_name(""), None);
    }
}
