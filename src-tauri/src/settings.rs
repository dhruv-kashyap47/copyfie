//! Minimal settings file (`settings.json`).
//!
//! M1-09 needs exactly one persisted value: the global shortcut. M1-10 extends
//! this file with retention, theme, blocklist, and the rest. Unknown fields are
//! preserved so a settings file written by another build is not destroyed.
//!
//! Write policy: temp file → fsync → rename, with a `.bak` copy of the previous
//! version. A parse error never overwrites the broken file; defaults are used
//! in memory and the user is warned.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const DEFAULT_HOTKEY: &str = "CommandOrControl+Shift+Space";
pub const DEFAULT_THEME: &str = "system";
pub const DEFAULT_ACCENT: &str = "#5B9DFF";
pub const DEFAULT_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_PALETTE_READY: &str = "auto";
pub const DEFAULT_PALETTE_IDLE_MINUTES: u32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_accent")]
    pub accent: String,
    /// 0 means "Forever".
    #[serde(default = "default_retention")]
    pub retention_days: u32,
    #[serde(default)]
    pub capture_paused: bool,
    /// `auto` (hybrid), `always`, or `never`.
    #[serde(default = "default_palette_ready")]
    pub palette_ready: String,
    /// Idle minutes before the palette webview is destroyed in `auto` mode.
    #[serde(default = "default_palette_idle")]
    pub palette_idle_minutes: u32,
    /// Fields owned by other/newer builds.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            hotkey: DEFAULT_HOTKEY.to_string(),
            theme: DEFAULT_THEME.to_string(),
            accent: DEFAULT_ACCENT.to_string(),
            retention_days: DEFAULT_RETENTION_DAYS,
            capture_paused: false,
            palette_ready: DEFAULT_PALETTE_READY.to_string(),
            palette_idle_minutes: DEFAULT_PALETTE_IDLE_MINUTES,
            extra: Map::new(),
        }
    }
}

impl Settings {
    /// Clamp and validate values that come from disk or IPC.
    pub fn sanitized(mut self) -> Self {
        self.theme = match self.theme.trim().to_ascii_lowercase().as_str() {
            "light" => "light".to_string(),
            "dark" => "dark".to_string(),
            _ => DEFAULT_THEME.to_string(),
        };
        self.accent = normalize_accent(&self.accent);
        self.retention_days = if self.retention_days == 0 {
            0
        } else {
            self.retention_days.clamp(1, 365)
        };
        self.palette_ready = match self.palette_ready.trim().to_ascii_lowercase().as_str() {
            "always" => "always".to_string(),
            "never" => "never".to_string(),
            _ => DEFAULT_PALETTE_READY.to_string(),
        };
        self.palette_idle_minutes = self.palette_idle_minutes.clamp(1, 120);
        self.hotkey = self.hotkey.trim().to_string();
        if self.hotkey.is_empty() {
            self.hotkey = DEFAULT_HOTKEY.to_string();
        }
        self
    }
}

fn normalize_accent(value: &str) -> String {
    let trimmed = value.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        format!("#{}", hex.to_ascii_uppercase())
    } else {
        DEFAULT_ACCENT.to_string()
    }
}

fn default_version() -> u32 {
    1
}

fn default_hotkey() -> String {
    DEFAULT_HOTKEY.to_string()
}

fn default_theme() -> String {
    DEFAULT_THEME.to_string()
}

fn default_accent() -> String {
    DEFAULT_ACCENT.to_string()
}

fn default_retention() -> u32 {
    DEFAULT_RETENTION_DAYS
}

fn default_palette_ready() -> String {
    DEFAULT_PALETTE_READY.to_string()
}

fn default_palette_idle() -> u32 {
    DEFAULT_PALETTE_IDLE_MINUTES
}

/// Plans do not need a settings directory abstraction: everything lives next to
/// the database under the app data root.
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join("settings.json"),
        }
    }

    pub fn load(&self) -> Settings {
        match fs::read_to_string(&self.path) {
            Ok(text) => match serde_json::from_str::<Settings>(&text) {
                Ok(settings) => settings,
                Err(error) => {
                    log::warn!(
                        target: "copyfie::settings",
                        "settings.json is invalid; using defaults without overwriting it: {error}"
                    );
                    Settings::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Err(error) => {
                log::warn!(
                    target: "copyfie::settings",
                    "could not read settings.json; using defaults: {error}"
                );
                Settings::default()
            }
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), String> {
        let settings = settings.clone().sanitized();
        let json = serde_json::to_string_pretty(&settings)
            .map_err(|error| format!("could not serialize settings: {error}"))?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("could not create the settings directory: {error}"))?;
        }

        let temp = self.path.with_extension("json.tmp");
        {
            let mut file = fs::File::create(&temp)
                .map_err(|error| format!("could not create settings.json.tmp: {error}"))?;
            file.write_all(json.as_bytes())
                .map_err(|error| format!("could not write settings.json.tmp: {error}"))?;
            file.sync_all()
                .map_err(|error| format!("could not flush settings.json.tmp: {error}"))?;
        }

        if self.path.exists() {
            let backup = self.path.with_extension("json.bak");
            if let Err(error) = fs::copy(&self.path, &backup) {
                log::warn!(target: "copyfie::settings", "could not keep settings.json.bak: {error}");
            }
        }

        fs::rename(&temp, &self.path)
            .map_err(|error| format!("could not replace settings.json: {error}"))?;
        Ok(())
    }

    pub fn save_hotkey(&self, hotkey: &str) -> Result<(), String> {
        let mut settings = self.load();
        settings.hotkey = hotkey.to_string();
        self.save(&settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn store() -> (tempfile::TempDir, SettingsStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = SettingsStore::new(dir.path());
        (dir, store)
    }

    #[test]
    fn defaults_when_missing() {
        let (_dir, store) = store();
        let settings = store.load();
        assert_eq!(settings.version, 1);
        assert_eq!(settings.hotkey, DEFAULT_HOTKEY);
    }

    #[test]
    fn round_trips_and_preserves_unknown_fields() {
        let (_dir, store) = store();
        let mut settings = Settings {
            hotkey: "CommandOrControl+Shift+F9".to_string(),
            ..Settings::default()
        };
        settings
            .extra
            .insert("future_setting".to_string(), json!(true));

        store.save(&settings).expect("save");
        let loaded = store.load();
        assert_eq!(loaded.hotkey, "CommandOrControl+Shift+F9");
        assert_eq!(loaded.extra.get("future_setting"), Some(&json!(true)));
    }

    #[test]
    fn broken_file_uses_defaults_and_is_not_overwritten() {
        let (dir, store) = store();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{ not json").expect("write");

        let settings = store.load();
        assert_eq!(settings.hotkey, DEFAULT_HOTKEY);
        assert_eq!(fs::read_to_string(&path).expect("read"), "{ not json");
    }

    #[test]
    fn save_keeps_a_backup_of_the_previous_file() {
        let (dir, store) = store();
        store
            .save_hotkey("CommandOrControl+Shift+F9")
            .expect("first");
        store
            .save_hotkey("CommandOrControl+Shift+F10")
            .expect("second");

        let backup = dir.path().join("settings.json.bak");
        let text = fs::read_to_string(backup).expect("backup");
        assert!(text.contains("F9"));
        assert!(fs::read_to_string(dir.path().join("settings.json"))
            .expect("main")
            .contains("F10"));
    }

    #[test]
    fn save_hotkey_keeps_other_values() {
        let (_dir, store) = store();
        let mut settings = Settings::default();
        settings.extra.insert("blocklist".to_string(), json!([]));
        store.save(&settings).expect("save");

        store
            .save_hotkey("CommandOrControl+Alt+Space")
            .expect("save hotkey");
        let loaded = store.load();
        assert_eq!(loaded.extra.get("blocklist"), Some(&json!([])));
    }

    #[test]
    fn sanitizes_theme_accent_and_retention() {
        let settings = Settings {
            theme: "LIGHT".to_string(),
            accent: " 1e5bd7 ".to_string(),
            retention_days: 9_999,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(settings.theme, "light");
        assert_eq!(settings.accent, "#1E5BD7");
        assert_eq!(settings.retention_days, 365);

        let fallback = Settings {
            theme: "neon".to_string(),
            accent: "not-a-color".to_string(),
            retention_days: 0,
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(fallback.theme, "system");
        assert_eq!(fallback.accent, DEFAULT_ACCENT);
        assert_eq!(fallback.retention_days, 0);
    }

    #[test]
    fn save_applies_sanitization() {
        let (dir, store) = store();
        store
            .save(&Settings {
                theme: "garbage".to_string(),
                accent: "#zzzzzz".to_string(),
                ..Settings::default()
            })
            .expect("save");

        let text = fs::read_to_string(dir.path().join("settings.json")).expect("read");
        assert!(text.contains("\"theme\": \"system\""));
        assert!(text.contains(DEFAULT_ACCENT));
    }
}
