//! Item reads and writes.
//!
//! Dedupe is an atomic insert-or-update on `(kind, content_hash)`:
//! - new content inserts a row;
//! - a recopy bumps `captured_at`, refreshes the source app, resets
//!   `use_count`, preserves the payload, and keeps pins pinned.
//!
//! `is_new` is determined by which statement matched, not by comparing
//! timestamps, so a recopy in the same millisecond is still reported as an
//! existing row. `is_new` is the persistence acknowledgement that lets the
//! capture pipeline treat a failed write as retryable.

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Type, ValueRef};
use rusqlite::{params, OptionalExtension, Row};

use crate::model::{Item, ItemKind, ItemSummary, ListQuery, NewItem};
use crate::retention::{expiry_for, Retention};

use super::{Database, InsertOutcome, Result};

const ITEM_COLUMNS: &str = "id, kind, content_hash, text, preview, blob_name, thumb_name, \
     byte_size, width, height, source_app_id, source_app_name, source_window_title, \
     is_pinned, captured_at, first_seen_at, last_used_at, use_count, expires_at, ocr_text";

const INSERT_SQL: &str = "INSERT INTO items ( \
     kind, content_hash, text, preview, blob_name, thumb_name, byte_size, width, height, \
     source_app_id, source_app_name, source_window_title, captured_at, first_seen_at, expires_at \
     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13, ?14) \
     ON CONFLICT(kind, content_hash) DO NOTHING \
     RETURNING id";

const UPDATE_SQL: &str = "UPDATE items SET \
       captured_at = ?3, \
       source_app_id = ?4, \
       source_app_name = ?5, \
       source_window_title = ?6, \
       use_count = 0, \
       expires_at = CASE WHEN is_pinned = 1 THEN NULL ELSE ?7 END \
     WHERE kind = ?1 AND content_hash = ?2 \
     RETURNING id";

impl Database {
    /// Insert new content, or bump an existing row with the same identity.
    pub fn upsert(&self, item: &NewItem) -> Result<InsertOutcome> {
        let inserted: Option<i64> = self
            .conn()
            .query_row(
                INSERT_SQL,
                params![
                    item.kind,
                    item.content_hash.as_slice(),
                    item.text,
                    item.preview,
                    item.blob_name,
                    item.thumb_name,
                    item.byte_size,
                    item.width,
                    item.height,
                    item.source_app_id,
                    item.source_app_name,
                    item.source_window_title,
                    item.captured_at,
                    item.expires_at,
                ],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(id) = inserted {
            return Ok(InsertOutcome { id, is_new: true });
        }

        // The unique index matched: refresh recency/source and keep the payload.
        let id = self.conn().query_row(
            UPDATE_SQL,
            params![
                item.kind,
                item.content_hash.as_slice(),
                item.captured_at,
                item.source_app_id,
                item.source_app_name,
                item.source_window_title,
                item.expires_at,
            ],
            |row| row.get(0),
        )?;

        Ok(InsertOutcome { id, is_new: false })
    }

    /// Full row by id.
    pub fn get(&self, id: i64) -> Result<Option<Item>> {
        let sql = format!("SELECT {ITEM_COLUMNS} FROM items WHERE id = ?1");
        Ok(self
            .conn()
            .query_row(&sql, [id], item_from_row)
            .optional()?)
    }

    /// Newest-first listing for the palette, with pins first.
    pub fn list(&self, query: &ListQuery) -> Result<Vec<ItemSummary>> {
        let limit = i64::from(query.limit.clamp(1, 100));
        let offset = i64::from(query.offset);
        let order = "ORDER BY is_pinned DESC, captured_at DESC, id DESC";

        let sql = match (query.kind, query.pinned_only) {
            (Some(_), true) => format!(
                "SELECT {SUMMARY_COLUMNS} FROM items WHERE kind = ?1 AND is_pinned = 1 {order} LIMIT ?2 OFFSET ?3"
            ),
            (Some(_), false) => format!(
                "SELECT {SUMMARY_COLUMNS} FROM items WHERE kind = ?1 {order} LIMIT ?2 OFFSET ?3"
            ),
            (None, true) => format!(
                "SELECT {SUMMARY_COLUMNS} FROM items WHERE is_pinned = 1 {order} LIMIT ?1 OFFSET ?2"
            ),
            (None, false) => {
                format!("SELECT {SUMMARY_COLUMNS} FROM items {order} LIMIT ?1 OFFSET ?2")
            }
        };

        let mut stmt = self.conn().prepare(&sql)?;
        let rows = match (query.kind, query.pinned_only) {
            (Some(kind), _) => stmt.query_map(params![kind, limit, offset], summary_from_row)?,
            (None, _) => stmt.query_map(params![limit, offset], summary_from_row)?,
        };

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// Count rows, optionally restricted to one kind.
    pub fn count(&self, kind: Option<ItemKind>) -> Result<i64> {
        let count = match kind {
            Some(kind) => self.conn().query_row(
                "SELECT count(*) FROM items WHERE kind = ?1",
                [kind],
                |row| row.get(0),
            )?,
            None => self
                .conn()
                .query_row("SELECT count(*) FROM items", [], |row| row.get(0))?,
        };
        Ok(count)
    }

    /// Pin or unpin. Unpinning recomputes expiry and deletes a already-expired
    /// item immediately. Returns `false` when the id does not exist.
    pub fn set_pinned(
        &self,
        id: i64,
        pinned: bool,
        now: i64,
        retention: Retention,
    ) -> Result<bool> {
        let captured_at: Option<i64> = self
            .conn()
            .query_row("SELECT captured_at FROM items WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?;

        let Some(captured_at) = captured_at else {
            return Ok(false);
        };

        let expires_at = if pinned {
            None
        } else {
            expiry_for(captured_at, retention)
        };

        self.conn().execute(
            "UPDATE items SET is_pinned = ?2, expires_at = ?3 WHERE id = ?1",
            params![id, pinned, expires_at],
        )?;

        if !pinned {
            self.conn().execute(
                "DELETE FROM items \
                 WHERE id = ?1 AND is_pinned = 0 AND expires_at IS NOT NULL AND expires_at <= ?2",
                params![id, now],
            )?;
        }

        Ok(true)
    }

    /// Record a paste/copy use.
    pub fn touch_used(&self, id: i64, now: i64) -> Result<bool> {
        let changed = self.conn().execute(
            "UPDATE items SET use_count = use_count + 1, last_used_at = ?2 WHERE id = ?1",
            params![id, now],
        )?;
        Ok(changed > 0)
    }

    /// Delete one item. FTS rows follow through triggers.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let changed = self
            .conn()
            .execute("DELETE FROM items WHERE id = ?1", [id])?;
        Ok(changed > 0)
    }

    /// Delete history. `keep_pinned` implements "Clear history keeps pins".
    pub fn clear(&self, keep_pinned: bool) -> Result<usize> {
        let changed = if keep_pinned {
            self.conn()
                .execute("DELETE FROM items WHERE is_pinned = 0", [])?
        } else {
            self.conn().execute("DELETE FROM items", [])?
        };
        Ok(changed)
    }

    /// Read a `meta` value (trial anchor, schema flags, counters).
    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn()
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }

    /// Write a `meta` value.
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn().execute(
            "INSERT INTO meta(key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

const SUMMARY_COLUMNS: &str =
    "id, kind, preview, source_app_name, is_pinned, captured_at, thumb_name, byte_size, width, height";

pub(crate) fn summary_from_row(row: &Row<'_>) -> rusqlite::Result<ItemSummary> {
    Ok(ItemSummary {
        id: row.get(0)?,
        kind: row.get(1)?,
        preview: row.get(2)?,
        source_app_name: row.get(3)?,
        is_pinned: row.get(4)?,
        captured_at: row.get(5)?,
        thumb_name: row.get(6)?,
        byte_size: row.get(7)?,
        width: row.get(8)?,
        height: row.get(9)?,
    })
}

fn item_from_row(row: &Row<'_>) -> rusqlite::Result<Item> {
    let hash: Vec<u8> = row.get(2)?;
    Ok(Item {
        id: row.get(0)?,
        kind: row.get(1)?,
        content_hash: hash_from_vec(hash)?,
        text: row.get(3)?,
        preview: row.get(4)?,
        blob_name: row.get(5)?,
        thumb_name: row.get(6)?,
        byte_size: row.get(7)?,
        width: row.get(8)?,
        height: row.get(9)?,
        source_app_id: row.get(10)?,
        source_app_name: row.get(11)?,
        source_window_title: row.get(12)?,
        is_pinned: row.get(13)?,
        captured_at: row.get(14)?,
        first_seen_at: row.get(15)?,
        last_used_at: row.get(16)?,
        use_count: row.get(17)?,
        expires_at: row.get(18)?,
        ocr_text: row.get(19)?,
    })
}

fn hash_from_vec(bytes: Vec<u8>) -> rusqlite::Result<[u8; 32]> {
    let len = bytes.len();
    bytes.try_into().map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(2, Type::Blob, Box::new(InvalidHashLength(len)))
    })
}

#[derive(Debug, thiserror::Error)]
#[error("content_hash must be exactly 32 bytes, got {0}")]
struct InvalidHashLength(usize);

impl FromSql for ItemKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Integer(raw) => ItemKind::from_i64(raw).ok_or(FromSqlError::OutOfRange(raw)),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for ItemKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_i64()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceApp;

    fn db() -> Database {
        Database::open_in_memory().expect("open")
    }

    fn fts_ids(db: &Database, query: &str) -> Vec<i64> {
        let mut stmt = db
            .conn()
            .prepare("SELECT rowid FROM items_fts WHERE items_fts MATCH ?1 ORDER BY rowid")
            .expect("prepare");
        let rows = stmt
            .query_map([query], |row| row.get::<_, i64>(0))
            .expect("query");
        rows.map(|row| row.expect("row")).collect()
    }

    #[test]
    fn fts_tracks_insert_update_and_delete() {
        let db = db();
        let first = NewItem::text(
            "the quick brown fox",
            SourceApp {
                name: Some("Notes".to_string()),
                ..SourceApp::default()
            },
            1,
            Retention::Days(30),
        );
        let id = db.upsert(&first).expect("insert").id;

        assert_eq!(fts_ids(&db, "\"quick\""), vec![id]);
        assert_eq!(fts_ids(&db, "qui*"), vec![id]);
        assert_eq!(fts_ids(&db, "\"notes\""), vec![id]);

        // A recopy from another app reindexes source_app_name.
        let recopy = NewItem::text(
            "the quick brown fox",
            SourceApp {
                name: Some("Browser".to_string()),
                ..SourceApp::default()
            },
            2,
            Retention::Days(30),
        );
        db.upsert(&recopy).expect("recopy");
        assert!(fts_ids(&db, "\"notes\"").is_empty());
        assert_eq!(fts_ids(&db, "\"browser\""), vec![id]);

        db.delete(id).expect("delete");
        assert!(fts_ids(&db, "\"quick\"").is_empty());
    }

    #[test]
    fn fts_follows_retention_cleanup() {
        let db = db();
        let item = NewItem::text("ephemeral", SourceApp::default(), 0, Retention::Days(7));
        let id = db.upsert(&item).expect("insert").id;
        assert_eq!(fts_ids(&db, "eph*"), vec![id]);

        db.delete_expired(7 * crate::retention::DAY_MS)
            .expect("cleanup");
        assert!(fts_ids(&db, "eph*").is_empty());
    }
}
