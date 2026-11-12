//! Synthetic paste input.
//!
//! `enigo` provides one cross-platform API (SendInput / CGEvent / XTEST).
//! macOS uses the raw virtual keycode 9 for V, not `Key::Unicode('v')` — the
//! plan documents a release-only crash in that path.

use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

use crate::{PasteInput, PlatformError};

pub struct SystemPasteInput;

impl PasteInput for SystemPasteInput {
    fn send_paste(&self) -> Result<(), PlatformError> {
        let settings = Settings {
            // Accessibility onboarding is M2-10; fail into copy-only instead of
            // surprising the user with a permission prompt mid-paste.
            open_prompt_to_get_permissions: false,
            ..Settings::default()
        };

        let mut enigo = Enigo::new(&settings)
            .map_err(|error| PlatformError::Other(format!("input backend unavailable: {error}")))?;

        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = Key::Control;

        #[cfg(target_os = "macos")]
        let paste_key = Key::Other(9);
        #[cfg(not(target_os = "macos"))]
        let paste_key = Key::Unicode('v');

        enigo
            .key(modifier, Press)
            .map_err(|error| PlatformError::Other(error.to_string()))?;

        let clicked = enigo
            .key(paste_key, Click)
            .map_err(|error| PlatformError::Other(error.to_string()));
        let released = enigo
            .key(modifier, Release)
            .map_err(|error| PlatformError::Other(error.to_string()));

        // Always release the modifier, even when the key click failed.
        clicked?;
        released?;
        Ok(())
    }
}
