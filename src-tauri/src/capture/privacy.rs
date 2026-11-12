//! Privacy gate and blocklist matching.
//!
//! The gate runs on formats only: it decides whether the payload may be read at
//! all. Order: paused → concealed → blocked app → has text. Pause is first
//! because it avoids pointless platform calls; the rest matches PLAN §3.

use copyfie_core::{Retention, SourceApp};
use copyfie_platform::ClipboardProbe;

use crate::capture::source_app::basename;

/// Blocklist semantics differ by OS; store the owning platform with the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    pub const fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(target_os = "macos")]
        {
            Platform::MacOs
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            Platform::Linux
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::MacOs => "macos",
            Platform::Linux => "linux",
        }
    }
}

/// One "never capture from this app" rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlocklistEntry {
    pub platform: Platform,
    /// Windows: full exe path or basename. macOS: bundle identifier.
    /// Linux: WM_CLASS or executable basename.
    pub id: String,
    pub label: Option<String>,
}

/// Mutable capture settings shared between the pipeline and the settings UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureConfig {
    pub paused: bool,
    pub skip_concealed: bool,
    pub retention: Retention,
    pub blocklist: Vec<BlocklistEntry>,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            paused: false,
            skip_concealed: true,
            retention: Retention::default(),
            blocklist: Vec::new(),
        }
    }
}

/// Why a change was not stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    Paused,
    Concealed,
    BlockedApp,
    NoText,
    EmptyText,
    Echo,
}

impl SkipReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            SkipReason::Paused => "paused",
            SkipReason::Concealed => "concealed",
            SkipReason::BlockedApp => "blocked_app",
            SkipReason::NoText => "no_text",
            SkipReason::EmptyText => "empty_text",
            SkipReason::Echo => "echo",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    Allow,
    Skip(SkipReason),
}

pub struct PrivacyGate;

impl PrivacyGate {
    /// Decide whether this change may be read and stored.
    pub fn evaluate(
        config: &CaptureConfig,
        probe: &ClipboardProbe,
        source: Option<&SourceApp>,
        platform: Platform,
    ) -> GateDecision {
        if config.paused {
            return GateDecision::Skip(SkipReason::Paused);
        }

        if config.skip_concealed && probe.is_concealed {
            return GateDecision::Skip(SkipReason::Concealed);
        }

        if let Some(app_id) = source.and_then(|source| source.id.as_deref()) {
            let blocked = config
                .blocklist
                .iter()
                .any(|entry| entry.platform == platform && matches_blocklist_entry(entry, app_id));
            if blocked {
                return GateDecision::Skip(SkipReason::BlockedApp);
            }
        }

        if !probe.has_text {
            return GateDecision::Skip(SkipReason::NoText);
        }

        GateDecision::Allow
    }
}

/// Match one rule against the source app identity.
///
/// - Windows: case-insensitive full path or basename.
/// - macOS: exact bundle identifier.
/// - Linux: exact `WM_CLASS`, or executable basename.
pub fn matches_blocklist_entry(entry: &BlocklistEntry, app_id: &str) -> bool {
    let app_id = app_id.trim();
    let rule = entry.id.trim();
    if app_id.is_empty() || rule.is_empty() {
        return false;
    }

    match entry.platform {
        Platform::Windows => {
            let app_id = app_id.replace('/', "\\");
            let rule = rule.replace('/', "\\");
            app_id.eq_ignore_ascii_case(&rule)
                || basename(&app_id).eq_ignore_ascii_case(basename(&rule))
        }
        Platform::MacOs => app_id == rule,
        Platform::Linux => app_id == rule || basename(app_id) == rule,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(platform: Platform, id: &str) -> BlocklistEntry {
        BlocklistEntry {
            platform,
            id: id.to_string(),
            label: None,
        }
    }

    fn config_with(entries: Vec<BlocklistEntry>) -> CaptureConfig {
        CaptureConfig {
            blocklist: entries,
            ..CaptureConfig::default()
        }
    }

    #[test]
    fn windows_matches_full_path_and_basename_case_insensitively() {
        let full = entry(
            Platform::Windows,
            r"C:\Program Files\KeePassXC\KeePassXC.exe",
        );
        let base = entry(Platform::Windows, "keepassxc.exe");

        assert!(matches_blocklist_entry(
            &full,
            r"c:\program files\keepassxc\KEEpassXC.EXE"
        ));
        assert!(matches_blocklist_entry(&base, r"C:\Tools\KeePassXC.exe"));
        assert!(!matches_blocklist_entry(&base, r"C:\Tools\NotKeePass.exe"));
        // Slash normalization.
        assert!(matches_blocklist_entry(
            &entry(Platform::Windows, r"C:/Apps/Bitwarden.exe"),
            r"C:\Apps\Bitwarden.exe"
        ));
    }

    #[test]
    fn macos_matches_bundle_id_exactly() {
        let rule = entry(Platform::MacOs, "com.1password.1password");
        assert!(matches_blocklist_entry(&rule, "com.1password.1password"));
        assert!(!matches_blocklist_entry(&rule, "com.1password.1Password"));
        assert!(!matches_blocklist_entry(
            &rule,
            "/Applications/1Password.app"
        ));
    }

    #[test]
    fn linux_matches_wm_class_or_executable_basename() {
        let rule = entry(Platform::Linux, "keepassxc");
        assert!(matches_blocklist_entry(&rule, "keepassxc"));
        assert!(matches_blocklist_entry(&rule, "/usr/bin/keepassxc"));
        assert!(!matches_blocklist_entry(&rule, "KeepassXC"));
    }

    #[test]
    fn rules_are_platform_scoped() {
        let rule = entry(Platform::MacOs, "com.1password.1password");
        assert!(matches_blocklist_entry(&rule, "com.1password.1password"));
        // The same id on Linux is a different rule namespace.
        assert!(!matches_blocklist_entry(
            &entry(Platform::Linux, "com.1password.1password"),
            "keepassxc"
        ));
    }

    #[test]
    fn pause_wins_and_prevents_reads() {
        let config = CaptureConfig {
            paused: true,
            ..CaptureConfig::default()
        };
        assert_eq!(
            PrivacyGate::evaluate(&config, &ClipboardProbe::text(), None, Platform::Linux),
            GateDecision::Skip(SkipReason::Paused)
        );
    }

    #[test]
    fn concealed_beats_blocklist() {
        let source = SourceApp {
            id: Some("keepassxc".to_string()),
            ..SourceApp::default()
        };
        let config = config_with(vec![entry(Platform::Linux, "keepassxc")]);
        let probe = ClipboardProbe {
            is_concealed: true,
            ..ClipboardProbe::text()
        };

        assert_eq!(
            PrivacyGate::evaluate(&config, &probe, Some(&source), Platform::Linux),
            GateDecision::Skip(SkipReason::Concealed)
        );
    }

    #[test]
    fn blocklist_blocks_even_without_a_concealed_marker() {
        let source = SourceApp {
            id: Some("/usr/bin/keepassxc".to_string()),
            ..SourceApp::default()
        };
        let config = config_with(vec![entry(Platform::Linux, "keepassxc")]);

        assert_eq!(
            PrivacyGate::evaluate(
                &config,
                &ClipboardProbe::text(),
                Some(&source),
                Platform::Linux
            ),
            GateDecision::Skip(SkipReason::BlockedApp)
        );
    }

    #[test]
    fn non_text_changes_are_skipped_in_the_text_only_milestone() {
        assert_eq!(
            PrivacyGate::evaluate(
                &CaptureConfig::default(),
                &ClipboardProbe::default(),
                None,
                Platform::Windows
            ),
            GateDecision::Skip(SkipReason::NoText)
        );
    }

    #[test]
    fn allows_plain_text() {
        assert_eq!(
            PrivacyGate::evaluate(
                &CaptureConfig::default(),
                &ClipboardProbe::text(),
                None,
                Platform::Windows
            ),
            GateDecision::Allow
        );
    }

    #[test]
    fn skip_concealed_can_be_disabled() {
        let config = CaptureConfig {
            skip_concealed: false,
            ..CaptureConfig::default()
        };
        let probe = ClipboardProbe {
            is_concealed: true,
            ..ClipboardProbe::text()
        };
        assert_eq!(
            PrivacyGate::evaluate(&config, &probe, None, Platform::Windows),
            GateDecision::Allow
        );
    }
}
