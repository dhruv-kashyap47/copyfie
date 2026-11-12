//! Forward-only migrations driven by `PRAGMA user_version`.
//!
//! Rules:
//! - Never edit a shipped migration; append a new file.
//! - Each migration runs inside a transaction together with its version bump.
//! - If the file was written by a newer build, refuse to open it rather than
//!   risk corrupting data.

use rusqlite::Connection;

use super::{Result, StoreError};

/// `(name, sql)` in application order. `user_version` equals the index + 1.
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../../migrations/0001_init.sql")),
    ("0002_fts", include_str!("../../migrations/0002_fts.sql")),
    (
        "0003_indexes",
        include_str!("../../migrations/0003_indexes.sql"),
    ),
];

/// Schema version this build understands.
pub fn supported_version() -> i64 {
    MIGRATIONS.len() as i64
}

/// Schema version currently stored in the database.
pub fn current_version(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

pub(super) fn migrate(conn: &Connection) -> Result<()> {
    let found = current_version(conn)?;
    let supported = supported_version();

    if found > supported {
        return Err(StoreError::FutureSchema { found, supported });
    }

    if found == supported {
        return Ok(());
    }

    // `transaction()` needs `&mut Connection`; all callers open exclusively in
    // `Database::open`, so a short-lived unchecked transaction is not needed.
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let result = apply(conn, found as usize);
    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(err)
        }
    }
}

fn apply(conn: &Connection, from: usize) -> Result<()> {
    for (index, (name, sql)) in MIGRATIONS.iter().enumerate().skip(from) {
        conn.execute_batch(sql)
            .map_err(|source| StoreError::Migration { name, source })?;
        conn.pragma_update(None, "user_version", (index + 1) as i64)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    #[test]
    fn fresh_database_is_fully_migrated() {
        let db = Database::open_in_memory().expect("open");
        assert_eq!(
            current_version(db.conn()).expect("version"),
            supported_version()
        );
    }

    #[test]
    fn migration_is_idempotent_across_reopen() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("clipboard.db");

        drop(Database::open(&path).expect("first open"));
        let db = Database::open(&path).expect("second open");
        assert_eq!(
            current_version(db.conn()).expect("version"),
            supported_version()
        );
    }

    #[test]
    fn refuses_a_newer_schema() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("clipboard.db");

        drop(Database::open(&path).expect("open"));

        {
            let conn = rusqlite::Connection::open(&path).expect("raw open");
            conn.pragma_update(None, "user_version", 9_999_i64)
                .expect("bump");
        }

        match Database::open(&path) {
            Err(StoreError::FutureSchema { found, supported }) => {
                assert_eq!(found, 9_999);
                assert_eq!(supported, supported_version());
            }
            Err(other) => panic!("expected FutureSchema, got {other}"),
            Ok(_) => panic!("expected FutureSchema, but the database opened"),
        }
    }
}
