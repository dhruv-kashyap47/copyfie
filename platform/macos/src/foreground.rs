//! Foreground app identity on macOS via `NSWorkspace`.

use copyfie_core::SourceApp;
use copyfie_platform::ForegroundApp;
use objc2::rc::autoreleasepool;
use objc2_app_kit::NSWorkspace;

pub struct MacForeground;

impl ForegroundApp for MacForeground {
    fn current(&self) -> Option<SourceApp> {
        autoreleasepool(|_| {
            let workspace = NSWorkspace::sharedWorkspace();
            let application = workspace.frontmostApplication()?;

            Some(SourceApp {
                id: application.bundleIdentifier().map(|id| id.to_string()),
                name: application.localizedName().map(|name| name.to_string()),
                // Available only with Accessibility permission; that flow
                // belongs to paste (M1-08/M2-10), not capture.
                window_title: None,
            })
        })
    }
}
