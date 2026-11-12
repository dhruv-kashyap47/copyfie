//! FTS5 full-text search.
//!
//! The palette uses two layers: an instant in-memory filter over loaded
//! previews (frontend) and this authoritative FTS query. User input is never
//! interpolated into `MATCH`: every token is quoted, internal quotes are
//! doubled, and prefix matching uses FTS5's `"token"*` form.

mod query;

pub use query::{build_match, MAX_QUERY_CHARS};

use rusqlite::params;

use crate::model::{ItemKind, ItemSummary};
use crate::store::{Database, Result};

/// Hard cap for one search response.
pub const MAX_RESULTS: u32 = 100;

impl Database {
    /// Ranked full-text search. `query` is the raw user input.
    pub fn search(
        &self,
        query: &str,
        kind: Option<ItemKind>,
        limit: u32,
    ) -> Result<Vec<ItemSummary>> {
        let Some(match_query) = build_match(query) else {
            return Ok(Vec::new());
        };

        let limit = i64::from(limit.clamp(1, MAX_RESULTS));

        let mut statement = self.conn().prepare(
            "SELECT i.id, i.kind, i.preview, i.source_app_name, i.is_pinned, i.captured_at, \
                    i.thumb_name, i.byte_size, i.width, i.height \
             FROM items_fts f \
             JOIN items i ON i.id = f.rowid \
             WHERE items_fts MATCH ?1 \
               AND (?2 IS NULL OR i.kind = ?2) \
             ORDER BY i.is_pinned DESC, bm25(items_fts, 8.0, 3.0, 1.0), i.captured_at DESC \
             LIMIT ?3",
        )?;

        let rows = statement.query_map(
            params![match_query, kind, limit],
            crate::store::summary_from_row,
        )?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewItem, SourceApp};
    use crate::retention::Retention;

    const T0: i64 = 1_767_225_600_000;

    fn db() -> Database {
        Database::open_in_memory().expect("open")
    }

    fn insert(db: &Database, text: &str, at: i64) -> i64 {
        db.upsert(&NewItem::text(
            text,
            SourceApp::default(),
            at,
            Retention::Days(30),
        ))
        .expect("insert")
        .id
    }

    #[test]
    fn finds_words_and_prefixes() {
        let db = db();
        let id = insert(&db, "the quick brown fox", T0);

        assert_eq!(db.search("quick", None, 50).expect("search")[0].id, id);
        assert_eq!(db.search("qui", None, 50).expect("search")[0].id, id);
        assert_eq!(db.search("bro", None, 50).expect("search")[0].id, id);
        assert!(db.search("zebra", None, 50).expect("search").is_empty());
    }

    #[test]
    fn searches_preview_and_source_app_name() {
        let db = db();
        let id = db
            .upsert(&NewItem::text(
                "rule 34 applies",
                SourceApp {
                    name: Some("Chrome".to_string()),
                    ..SourceApp::default()
                },
                T0,
                Retention::Days(30),
            ))
            .expect("insert")
            .id;

        assert_eq!(db.search("chrome", None, 50).expect("search")[0].id, id);
        assert_eq!(db.search("rule", None, 50).expect("search")[0].id, id);
    }

    #[test]
    fn pinned_results_come_first() {
        let db = db();
        let ordinary = insert(&db, "shared term ordinary", T0);
        let pinned = insert(&db, "shared term pinned", T0 + 1);

        db.set_pinned(pinned, true, T0 + 2, Retention::Days(30))
            .expect("pin");

        let results = db.search("shared", None, 50).expect("search");
        assert_eq!(results[0].id, pinned);
        assert!(results[0].is_pinned);
        assert_eq!(results[1].id, ordinary);
    }

    #[test]
    fn filters_by_kind() {
        let db = db();
        insert(&db, "text match", T0);
        let files = NewItem::files(
            &["C:\\match.txt"],
            SourceApp::default(),
            T0 + 1,
            Retention::Days(30),
        );
        let files_id = db.upsert(&files).expect("files").id;

        let text_results = db.search("match", Some(ItemKind::Text), 50).expect("text");
        assert_eq!(text_results.len(), 1);
        assert_eq!(text_results[0].kind, ItemKind::Text);

        let file_results = db
            .search("match", Some(ItemKind::Files), 50)
            .expect("files");
        assert_eq!(file_results.len(), 1);
        assert_eq!(file_results[0].id, files_id);
    }

    #[test]
    fn supports_cjk_and_emoji_without_error() {
        let db = db();
        let id = insert(&db, "日本語のテスト", T0);

        assert_eq!(db.search("日本語", None, 50).expect("search")[0].id, id);
        // Emoji are not tokenized by unicode61; the local filter handles them,
        // and FTS must not error.
        assert!(db.search("🌍", None, 50).expect("search").is_empty());
    }

    #[test]
    fn injection_and_punctuation_are_safe() {
        let db = db();
        insert(&db, "safe content", T0);

        for query in [
            "x\" OR 1=1 --",
            "'; DROP TABLE items; --",
            "NEAR(",
            "*",
            "\"unterminated",
            "a AND b OR c",
        ] {
            let result = db.search(query, None, 50);
            assert!(result.is_ok(), "query {query:?} must not error");
        }

        assert_eq!(db.count(None).expect("count"), 1, "tables survive");
    }

    #[test]
    fn honors_the_limit_and_caps_it() {
        let db = db();
        for index in 0..10 {
            insert(&db, &format!("needle {index}"), T0 + index);
        }

        assert_eq!(db.search("needle", None, 3).expect("search").len(), 3);
        assert_eq!(db.search("needle", None, 1_000).expect("search").len(), 10);
    }

    #[test]
    fn empty_query_returns_nothing() {
        let db = db();
        insert(&db, "anything", T0);
        assert!(db.search("   ", None, 50).expect("search").is_empty());
    }
}
