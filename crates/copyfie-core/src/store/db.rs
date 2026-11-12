//! Connection setup and pragmas.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::migrate::migrate;
use super::Result;

/// A migrated SQLite database.
pub struct Database {
    conn: Connection,
    path: Option<PathBuf>,
}

impl Database {
    /// Open (or create) the database at `path`, apply pragmas, and migrate.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)?;
        configure(&conn, false)?;
        migrate(&conn)?;
        Ok(Self {
            conn,
            path: Some(path),
        })
    }

    /// In-memory database for tests and benches.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        configure(&conn, true)?;
        migrate(&conn)?;
        Ok(Self { conn, path: None })
    }

    /// Path on disk, if this is not an in-memory database.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

fn configure(conn: &Connection, in_memory: bool) -> Result<()> {
    // WAL is the durability/concurrency baseline. In-memory databases report
    // "memory" and ignore it.
    let _mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;

    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    conn.pragma_update(None, "cache_size", -8192_i64)?;
    conn.pragma_update(None, "mmap_size", 67_108_864_i64)?;
    conn.pragma_update(None, "wal_autocheckpoint", 256_i64)?;
    // Deleted text is zeroed on disk. Images are files and are removed by GC.
    conn.pragma_update(None, "secure_delete", "ON")?;

    if !in_memory {
        enable_incremental_autovacuum(conn)?;
    }

    Ok(())
}

/// `auto_vacuum` only takes effect for a fresh database, or after a `VACUUM`.
/// Run that one-time `VACUUM` when upgrading an existing file.
fn enable_incremental_autovacuum(conn: &Connection) -> Result<()> {
    const INCREMENTAL: i64 = 2;
    let current: i64 = conn.query_row("PRAGMA auto_vacuum", [], |row| row.get(0))?;
    if current == INCREMENTAL {
        return Ok(());
    }

    conn.pragma_update(None, "auto_vacuum", INCREMENTAL)?;

    let existing: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type IN ('table', 'index')",
        [],
        |row| row.get(0),
    )?;
    if existing > 0 {
        conn.execute_batch("VACUUM;")?;
    }

    Ok(())
}
