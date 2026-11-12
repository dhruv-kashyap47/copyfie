//! KDE password-manager hint detection.
//!
//! KeePassXC and KWallet set `x-kde-passwordManagerHint` on the selection. The
//! hint carries a value (`secret` or `clipboard`), but `clipboard-rs` exposes
//! only the MIME list, so presence is treated as concealed. That is
//! conservative: it may skip a non-secret item rather than risk storing one.

use copyfie_platform::system::ConcealedProbe;

const MARKER: &str = "x-kde-passwordManagerHint";

pub struct LinuxConcealed;

impl ConcealedProbe for LinuxConcealed {
    fn is_concealed(&self, formats: &[String]) -> bool {
        formats
            .iter()
            .any(|format| format.eq_ignore_ascii_case(MARKER))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_the_kde_hint_case_insensitively() {
        let probe = LinuxConcealed;
        assert!(probe.is_concealed(&[
            "text/plain".to_string(),
            "x-kde-passwordManagerHint".to_string()
        ]));
        assert!(probe.is_concealed(&["X-KDE-PASSWORDMANAGERHINT".to_string()]));
    }

    #[test]
    fn plain_text_is_not_concealed() {
        let probe = LinuxConcealed;
        assert!(!probe.is_concealed(&["text/plain".to_string(), "UTF8_STRING".to_string()]));
        assert!(!probe.is_concealed(&[]));
    }
}
