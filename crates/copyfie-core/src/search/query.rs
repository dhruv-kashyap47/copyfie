//! Query tokenization for FTS5 `MATCH`.
//!
//! Rules:
//! - Trim and cap the query (default 200 chars).
//! - Split on whitespace; each token becomes a quoted prefix query.
//! - Double embedded quotes, so no token can terminate the quoted phrase.
//! - Drop tokens with no letters/numbers (punctuation-only) because FTS5
//!   treats them as syntax or noise.
//!
//! The result is always a sequence of terms, never raw user input.

/// Longest query accepted from the UI.
pub const MAX_QUERY_CHARS: usize = 200;

pub fn build_match(query: &str) -> Option<String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }

    let capped: String = trimmed.chars().take(MAX_QUERY_CHARS).collect();
    let mut terms = Vec::new();

    for token in capped.split_whitespace() {
        let escaped = token.replace('"', "\"\"");
        let has_content = escaped
            .chars()
            .any(|character| character.is_alphanumeric() || !character.is_ascii());
        if has_content {
            terms.push(format!("\"{escaped}\"*"));
        }
    }

    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_quoted_prefix_terms() {
        assert_eq!(build_match("hello"), Some("\"hello\"*".to_string()));
        assert_eq!(
            build_match("hello world"),
            Some("\"hello\"* \"world\"*".to_string())
        );
        assert_eq!(
            build_match("  spaced  out  "),
            Some("\"spaced\"* \"out\"*".to_string())
        );
    }

    #[test]
    fn doubles_quotes_so_input_cannot_escape_the_phrase() {
        let built = build_match("say \"hi\"").expect("query");
        assert_eq!(built, "\"say\"* \"\"\"hi\"\"\"*");

        // The generated string must still be valid to FTS5 (exercised in the
        // store tests); structurally, every term is quoted.
        assert!(built.contains("\"\"\"hi\"\"\""));
    }

    #[test]
    fn drops_punctuation_only_tokens() {
        assert_eq!(build_match("!!!"), None);
        assert_eq!(build_match("  ...  "), None);
        assert_eq!(build_match(""), None);
        assert_eq!(build_match("   "), None);
    }

    #[test]
    fn keeps_unicode_and_emoji() {
        assert_eq!(build_match("日本語"), Some("\"日本語\"*".to_string()));
        let emoji = build_match("🌍").expect("emoji");
        assert!(emoji.starts_with('"'));
    }

    #[test]
    fn caps_long_queries() {
        let query = "a".repeat(MAX_QUERY_CHARS + 500);
        let built = build_match(&query).expect("query");
        assert_eq!(built.matches('"').count(), 2);
        assert_eq!(built.len(), MAX_QUERY_CHARS + 3);
    }

    #[test]
    fn injection_attempts_stay_inside_quotes() {
        let built = build_match("x\" OR 1=1 --").expect("query");
        assert!(!built.contains(" OR 1=1 "));
        assert!(built.starts_with('"'));
        assert_eq!(built.matches('"').count() % 2, 0);
    }
}
