//! Data model for clipboard items.
//!
//! These types are shared by the store, the search layer, and (via serde) the
//! Tauri IPC boundary. Timestamps are Unix milliseconds; formatting happens in
//! the frontend.

use serde::Serialize;

use crate::hash;
use crate::retention::{expiry_for, Retention};

/// What a captured item contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Text = 0,
    Image = 1,
    Files = 2,
}

impl ItemKind {
    pub fn as_i64(self) -> i64 {
        self as i64
    }

    pub fn from_i64(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Text),
            1 => Some(Self::Image),
            2 => Some(Self::Files),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Files => "files",
        }
    }
}

/// Number of characters kept in the searchable/display preview.
pub const PREVIEW_CHARS: usize = 400;

/// First [`PREVIEW_CHARS`] characters of `text`, never splitting a code point.
pub fn make_preview(text: &str) -> String {
    text.chars().take(PREVIEW_CHARS).collect()
}

/// The application that owned the foreground window when the item was copied.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceApp {
    /// Stable identity used by the blocklist: exe path, bundle id, or WM_CLASS.
    pub id: Option<String>,
    /// Friendly display name.
    pub name: Option<String>,
    /// Foreground window title at capture time.
    pub window_title: Option<String>,
}

/// A row of `items`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: i64,
    pub kind: ItemKind,
    pub content_hash: [u8; 32],
    pub text: String,
    pub preview: String,
    pub blob_name: Option<String>,
    pub thumb_name: Option<String>,
    pub byte_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub source_app_id: Option<String>,
    pub source_app_name: Option<String>,
    pub source_window_title: Option<String>,
    pub is_pinned: bool,
    pub captured_at: i64,
    pub first_seen_at: i64,
    pub last_used_at: Option<i64>,
    pub use_count: i64,
    pub expires_at: Option<i64>,
    pub ocr_text: Option<String>,
}

/// The subset the palette renders and filters locally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: i64,
    pub kind: ItemKind,
    pub preview: String,
    pub source_app_name: Option<String>,
    pub is_pinned: bool,
    pub captured_at: i64,
    pub thumb_name: Option<String>,
    pub byte_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

/// A validated, normalized item ready to be inserted.
#[derive(Debug, Clone)]
pub struct NewItem {
    pub kind: ItemKind,
    pub content_hash: [u8; 32],
    pub text: String,
    pub preview: String,
    pub blob_name: Option<String>,
    pub thumb_name: Option<String>,
    pub byte_size: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub source_app_id: Option<String>,
    pub source_app_name: Option<String>,
    pub source_window_title: Option<String>,
    pub captured_at: i64,
    pub expires_at: Option<i64>,
}

impl NewItem {
    /// Build a text item from a raw capture.
    ///
    /// The caller supplies the effective clock; expiry comes from the user's
    /// retention setting (0 = Forever).
    pub fn text(raw_text: &str, source: SourceApp, captured_at: i64, retention: Retention) -> Self {
        let text = hash::canonicalize_text(raw_text);
        let preview = make_preview(&text);
        Self {
            kind: ItemKind::Text,
            content_hash: hash::hash_bytes(text.as_bytes()),
            byte_size: text.len() as i64,
            expires_at: expiry_for(captured_at, retention),
            text,
            preview,
            blob_name: None,
            thumb_name: None,
            width: None,
            height: None,
            source_app_id: source.id,
            source_app_name: source.name,
            source_window_title: source.window_title,
            captured_at,
        }
    }

    /// Build a files item. `paths` must already be normalized by the platform
    /// layer; hashing sorts and trims them again for safety.
    pub fn files<S: AsRef<str>>(
        paths: &[S],
        source: SourceApp,
        captured_at: i64,
        retention: Retention,
    ) -> Self {
        let normalized: Vec<String> = paths
            .iter()
            .map(|path| hash::normalize_path(path.as_ref()))
            .filter(|path| !path.is_empty())
            .collect();
        let text = serde_json::to_string(&normalized).unwrap_or_else(|_| "[]".to_string());
        Self {
            kind: ItemKind::Files,
            content_hash: hash::hash_files(&normalized),
            preview: make_preview(&normalized.join("\n")),
            byte_size: text.len() as i64,
            expires_at: expiry_for(captured_at, retention),
            text,
            blob_name: None,
            thumb_name: None,
            width: None,
            height: None,
            source_app_id: source.id,
            source_app_name: source.name,
            source_window_title: source.window_title,
            captured_at,
        }
    }
}

/// Filter and paging options for a history listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListQuery {
    pub limit: u32,
    pub offset: u32,
    pub kind: Option<ItemKind>,
    pub pinned_only: bool,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
            kind: None,
            pinned_only: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_is_char_safe() {
        let text = "🌍".repeat(PREVIEW_CHARS + 50);
        let preview = make_preview(&text);
        assert_eq!(preview.chars().count(), PREVIEW_CHARS);
        assert!(text.starts_with(&preview));
    }

    #[test]
    fn text_item_normalizes_and_hashes() {
        let item = NewItem::text(
            "hello\r\nworld\0",
            SourceApp::default(),
            1_000,
            Retention::Days(30),
        );
        assert_eq!(item.text, "hello\nworld");
        assert_eq!(item.content_hash, hash::hash_text("hello\nworld"));
        assert_eq!(item.expires_at, Some(1_000 + 30 * 86_400_000));
        assert_eq!(item.byte_size, 11);
    }

    #[test]
    fn forever_retention_never_expires() {
        let item = NewItem::text("x", SourceApp::default(), 1, Retention::Forever);
        assert_eq!(item.expires_at, None);
    }

    #[test]
    fn files_item_dedupes_order() {
        let a = NewItem::files(
            &["C:\\b.txt", "C:\\a.txt"],
            SourceApp::default(),
            1,
            Retention::Forever,
        );
        let b = NewItem::files(
            &["C:\\a.txt", "C:\\b.txt"],
            SourceApp::default(),
            2,
            Retention::Forever,
        );
        assert_eq!(a.content_hash, b.content_hash);
        // Display order is preserved; only the hash is order-insensitive.
        assert_eq!(a.text, r#"["C:\\b.txt","C:\\a.txt"]"#);
    }

    #[test]
    fn kind_round_trips() {
        for kind in [ItemKind::Text, ItemKind::Image, ItemKind::Files] {
            assert_eq!(ItemKind::from_i64(kind.as_i64()), Some(kind));
        }
        assert_eq!(ItemKind::from_i64(9), None);
    }
}
