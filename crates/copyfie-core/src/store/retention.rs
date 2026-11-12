//! Age-based cleanup.
//!
//! Runs at startup, every five minutes, and after a write only when a
//! buffered threshold is crossed. One indexed `DELETE`; triggers keep FTS in
//! sync. Pinned rows are never eligible because their `expires_at` is NULL.

use super::{Database, Result};

impl Database {
    /// Delete every unpinned item whose expiry has passed. Returns the count.
    pub fn delete_expired(&self, now: i64) -> Result<usize> {
        let changed = self.conn().execute(
            "DELETE FROM items \
             WHERE is_pinned = 0 AND expires_at IS NOT NULL AND expires_at <= ?1",
            [now],
        )?;
        Ok(changed)
    }

    /// Reclaim free pages after a large delete. Safe to call periodically.
    pub fn incremental_vacuum(&self) -> Result<()> {
        self.conn().execute_batch("PRAGMA incremental_vacuum;")?;
        Ok(())
    }
}
