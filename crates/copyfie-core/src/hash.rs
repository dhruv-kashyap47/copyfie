//! Content canonicalization and hashing.
//!
//! The hash is the dedupe identity of an item, so canonicalization must be
//! deterministic and platform-independent. Images hash raw RGBA plus
//! dimensions (added with image capture); text and file lists hash their
//! normalized representation.

use std::path::Path;

/// Normalize captured text before hashing and storing it.
///
/// - CRLF becomes LF, so Windows and Linux copies of the same content dedupe.
/// - Trailing NUL bytes are stripped; some Windows clipboard producers pad.
///
/// A lone `\r` is preserved: it is legitimate content and rewriting it would
/// change what the user copied.
pub fn canonicalize_text(input: &str) -> String {
    let normalized = input.replace("\r\n", "\n");
    normalized.trim_end_matches('\0').to_string()
}

/// BLAKE3-256 over raw bytes.
pub fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
    blake3::hash(bytes).into()
}

/// BLAKE3-256 over canonicalized text.
pub fn hash_text(text: &str) -> [u8; 32] {
    hash_bytes(canonicalize_text(text).as_bytes())
}

/// Normalize one filesystem path for hashing and storage.
///
/// Trims trailing separators, preserving filesystem roots (`/`, `C:\`, UNC
/// prefixes). Case is preserved: Windows comparisons are case-insensitive, but
/// a stable byte representation matters for dedupe and future sync.
pub fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    let root_len = root_len(trimmed);
    if trimmed.len() <= root_len {
        return trimmed.to_string();
    }
    trimmed.trim_end_matches(['/', '\\']).to_string()
}

fn root_len(path: &str) -> usize {
    let bytes = path.as_bytes();
    if bytes.is_empty() {
        return 0;
    }
    // POSIX root.
    if bytes == b"/" {
        return 1;
    }
    // Windows drive root: `C:\` or `C:/`.
    if bytes.len() == 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return 3;
    }
    // UNC share root: `\\server\share` — keep the leading `\\`, trim later.
    0
}

/// Hash an unordered set of file paths (a multi-file copy).
///
/// Paths are normalized, deduplicated, sorted, and joined with `\n` so the
/// same selection copied in a different order dedupes to one item.
pub fn hash_files<S: AsRef<str>>(paths: &[S]) -> [u8; 32] {
    let mut normalized: Vec<String> = paths
        .iter()
        .map(|path| normalize_path(path.as_ref()))
        .filter(|path| !path.is_empty())
        .collect();
    normalized.sort_unstable();
    normalized.dedup();
    hash_bytes(normalized.join("\n").as_bytes())
}

/// Hash a decoded image by pixels, not by encoder output.
pub fn hash_image_rgba(rgba: &[u8], width: u32, height: u32) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&width.to_le_bytes());
    hasher.update(&height.to_le_bytes());
    hasher.update(rgba);
    hasher.finalize().into()
}

/// Normalize input that came from an OS clipboard as a file URL/path.
pub fn normalize_path_buf(path: impl AsRef<Path>) -> String {
    normalize_path(&path.as_ref().to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_crlf_to_lf() {
        assert_eq!(canonicalize_text("a\r\nb\r\nc"), "a\nb\nc");
        assert_eq!(hash_text("a\r\nb"), hash_text("a\nb"));
    }

    #[test]
    fn strips_trailing_nuls_only() {
        assert_eq!(canonicalize_text("abc\0\0"), "abc");
        assert_eq!(canonicalize_text("a\0b"), "a\0b");
    }

    #[test]
    fn preserves_lone_carriage_return() {
        assert_eq!(canonicalize_text("a\rb"), "a\rb");
        assert_ne!(hash_text("a\rb"), hash_text("a\nb"));
    }

    #[test]
    fn hashes_unicode_and_empty() {
        assert_eq!(hash_text("héllo 🌍"), hash_text("héllo 🌍"));
        assert_ne!(hash_text("héllo 🌍"), hash_text("hello 🌍"));
        assert_eq!(hash_text(""), hash_bytes(b""));
    }

    #[test]
    fn trims_trailing_separators_but_keeps_roots() {
        assert_eq!(
            normalize_path("C:\\Users\\me\\Desktop\\"),
            "C:\\Users\\me\\Desktop"
        );
        assert_eq!(normalize_path("/home/me/notes/"), "/home/me/notes");
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("C:\\"), "C:\\");
        assert_eq!(normalize_path("  /tmp/a.txt  "), "/tmp/a.txt");
    }

    #[test]
    fn file_hash_is_order_insensitive() {
        let a = hash_files(&["C:\\b.txt", "C:\\a.txt\\"]);
        let b = hash_files(&["C:\\a.txt", "C:\\b.txt"]);
        assert_eq!(a, b);
        assert_ne!(a, hash_files(&["C:\\a.txt"]));
    }

    #[test]
    fn image_hash_depends_on_dimensions_and_pixels() {
        let pixels = [255u8, 0, 0, 255];
        assert_ne!(
            hash_image_rgba(&pixels, 1, 1),
            hash_image_rgba(&pixels, 2, 1)
        );
        assert_ne!(
            hash_image_rgba(&pixels, 1, 1),
            hash_image_rgba(&[0u8, 0, 0, 255], 1, 1)
        );
    }
}
