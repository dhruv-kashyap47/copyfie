//! Capture and restore the frontmost macOS application.

use copyfie_platform::{FocusBackend, FocusTarget, PlatformError};
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};

pub struct MacFocus;

impl FocusBackend for MacFocus {
    fn capture(&self) -> Option<FocusTarget> {
        autoreleasepool(|_| {
            let workspace = NSWorkspace::sharedWorkspace();
            let application = workspace.frontmostApplication()?;
            Some(FocusTarget::MacOs {
                pid: application.processIdentifier(),
            })
        })
    }

    fn restore(&self, target: &FocusTarget) -> Result<(), PlatformError> {
        let FocusTarget::MacOs { pid } = *target else {
            return Err(PlatformError::Other(
                "focus target does not belong to macOS".to_string(),
            ));
        };

        let application = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
            .ok_or_else(|| {
                PlatformError::Other("the target application is no longer running".to_string())
            })?;

        #[allow(deprecated)]
        let options = NSApplicationActivationOptions::ActivateAllWindows
            | NSApplicationActivationOptions::ActivateIgnoringOtherApps;
        application.activateWithOptions(options);

        Ok(())
    }

    fn is_active(&self, target: &FocusTarget) -> bool {
        let FocusTarget::MacOs { pid } = *target else {
            return false;
        };

        autoreleasepool(|_| {
            NSWorkspace::sharedWorkspace()
                .frontmostApplication()
                .map(|application| application.processIdentifier() == pid)
                .unwrap_or(false)
        })
    }
}
