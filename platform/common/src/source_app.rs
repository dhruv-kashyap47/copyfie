//! Source-app helpers shared by the platform backends and the capture pipeline.
//!
//! Capture stores a stable id (exe path, bundle id, or WM_CLASS), a friendly
//! name, and the window title. The UI shows the name plus a colored initial in
//! v1.0; icons arrive in v1.1.

use copyfie_core::SourceApp;

/// Last path component, accepting both separators.
pub fn basename(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    let start = trimmed
        .rfind(['/', '\\'])
        .map(|index| index + 1)
        .unwrap_or(0);
    &trimmed[start..]
}

/// Human-readable name: explicit name, else executable basename, else
/// "Unknown" (Wayland commonly has no source app at all).
pub fn display_name(source: &SourceApp) -> String {
    if let Some(name) = source
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        return name.to_string();
    }

    let base = source
        .id
        .as_deref()
        .map(basename)
        .map(str::trim)
        .filter(|name| !name.is_empty());

    match base {
        Some(name) => name.to_string(),
        None => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_basenames_on_both_platforms() {
        assert_eq!(basename(r"C:\Apps\Notepad.exe"), "Notepad.exe");
        assert_eq!(basename("C:/Apps/Notepad.exe"), "Notepad.exe");
        assert_eq!(basename("/usr/bin/code"), "code");
        assert_eq!(basename("code"), "code");
        assert_eq!(basename(""), "");
        assert_eq!(basename("/"), "");
        assert_eq!(basename(r"C:\Apps\"), "Apps");
    }

    #[test]
    fn prefers_the_friendly_name() {
        let source = SourceApp {
            id: Some(r"C:\Apps\Notepad.exe".to_string()),
            name: Some("Notepad".to_string()),
            window_title: None,
        };
        assert_eq!(display_name(&source), "Notepad");
    }

    #[test]
    fn falls_back_to_the_executable_name() {
        let source = SourceApp {
            id: Some("/usr/bin/keepassxc".to_string()),
            name: Some("   ".to_string()),
            window_title: None,
        };
        assert_eq!(display_name(&source), "keepassxc");
    }

    #[test]
    fn unknown_when_nothing_is_available() {
        assert_eq!(display_name(&SourceApp::default()), "Unknown");
    }
}
