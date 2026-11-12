//! Platform backend aliases for the shell.
//!
//! The capture runner selects its backend directly; commands need only the
//! clipboard writer, which stays behind this module so no Tauri code touches
//! an OS clipboard API.

#[cfg(target_os = "windows")]
pub use copyfie_platform_windows as backend;

#[cfg(target_os = "macos")]
pub use copyfie_platform_macos as backend;

#[cfg(target_os = "linux")]
pub use copyfie_platform_linux as backend;

pub fn write_text(text: &str) -> Result<(), copyfie_platform::PlatformError> {
    backend::write_text(text)
}
