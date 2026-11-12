//! Paste orchestration.
//!
//! Order matters (PLAN §3/§6):
//! 1. hide the palette;
//! 2. restore the window that was focused before the palette opened;
//! 3. poll until that window is actually active (bounded timeout);
//! 4. arm echo suppression, write the item to the clipboard, settle briefly;
//! 5. synthesize Ctrl/Cmd+V.
//!
//! If focus cannot be restored or input injection fails, the item still ends up
//! on the clipboard and the palette returns with a "copied" report — never a
//! blind paste into the wrong window.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use copyfie_platform::{FocusBackend, FocusTarget, PasteInput};

use crate::capture::Suppressor;

/// How long to wait for the target to regain focus.
const FOCUS_TIMEOUT: Duration = Duration::from_millis(700);
/// Delay between clipboard write and the synthetic chord.
const SETTLE: Duration = Duration::from_millis(25);

pub struct PasteService {
    focus: Box<dyn FocusBackend>,
    input: Box<dyn PasteInput>,
    target: Mutex<Option<FocusTarget>>,
}

impl PasteService {
    pub fn new() -> Self {
        Self {
            focus: new_focus(),
            input: new_paste(),
            target: Mutex::new(None),
        }
    }

    /// Capture the foreground target. Called just before the palette is shown.
    pub fn remember_target(&self) {
        *self.lock() = self.focus.capture();
    }

    /// Take the remembered target for one paste attempt.
    pub fn take_target(&self) -> Option<FocusTarget> {
        self.lock().take()
    }

    /// Restore focus and paste. Runs on a blocking thread.
    pub fn perform(
        &self,
        target: &FocusTarget,
        text: &str,
        suppressor: &Suppressor,
    ) -> Result<(), String> {
        log::info!(target: "copyfie::paste", "restoring paste target");
        self.focus
            .restore(target)
            .map_err(|error| error.to_string())?;

        let started = Instant::now();
        let deadline = started + FOCUS_TIMEOUT;
        while Instant::now() < deadline && !self.focus.is_active(target) {
            std::thread::sleep(Duration::from_millis(10));
        }

        if !self.focus.is_active(target) {
            return Err("the previous window did not regain focus".to_string());
        }
        log::info!(
            target: "copyfie::paste",
            "target active after {} ms",
            started.elapsed().as_millis()
        );

        // The window can be foreground a few milliseconds before its input
        // queue is ready; give the OS a beat before the chord.
        std::thread::sleep(Duration::from_millis(15));

        suppressor.arm_text(text, Instant::now());
        if let Err(error) = crate::platform::write_text(text) {
            suppressor.clear();
            return Err(error.to_string());
        }

        std::thread::sleep(SETTLE);

        self.input.send_paste().map_err(|error| error.to_string())?;
        log::info!(target: "copyfie::paste", "paste chord sent");
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<FocusTarget>> {
        self.target
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

impl Default for PasteService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
fn new_focus() -> Box<dyn FocusBackend> {
    copyfie_platform_windows::new_focus()
}

#[cfg(target_os = "macos")]
fn new_focus() -> Box<dyn FocusBackend> {
    copyfie_platform_macos::new_focus()
}

#[cfg(target_os = "linux")]
fn new_focus() -> Box<dyn FocusBackend> {
    copyfie_platform_linux::new_focus()
}

#[cfg(target_os = "windows")]
fn new_paste() -> Box<dyn PasteInput> {
    copyfie_platform_windows::new_paste()
}

#[cfg(target_os = "macos")]
fn new_paste() -> Box<dyn PasteInput> {
    copyfie_platform_macos::new_paste()
}

#[cfg(target_os = "linux")]
fn new_paste() -> Box<dyn PasteInput> {
    copyfie_platform_linux::new_paste()
}
