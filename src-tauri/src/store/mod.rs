//! Single-writer store actor.
//!
//! The SQLite connection is owned by one dedicated thread. Commands send
//! closures over a channel and await a one-shot reply; this serializes writes,
//! cleanup, and future blob GC without a `Mutex<Connection>` spread across
//! handlers.
//!
//! The actor is intentionally synchronous inside the thread. Awaiting callers
//! use `tokio::sync::oneshot`, so a slow query never blocks the async runtime.

use std::sync::mpsc::{self, Sender};
use std::thread;

use copyfie_core::{Database, StoreError};
use thiserror::Error;
use tokio::sync::oneshot;

type Job = Box<dyn FnOnce(&Database) + Send + 'static>;

#[derive(Debug, Error)]
pub enum ActorError {
    #[error("store actor is not running")]
    Stopped,

    #[error(transparent)]
    Store(#[from] StoreError),
}

pub type Result<T> = std::result::Result<T, ActorError>;

/// Cheap, cloneable handle to the store thread.
#[derive(Clone)]
pub struct StoreHandle {
    tx: Sender<Job>,
}

impl StoreHandle {
    /// Start the actor thread that owns `db` until every handle is dropped.
    pub fn spawn(db: Database) -> Self {
        let (tx, rx) = mpsc::channel::<Job>();

        thread::Builder::new()
            .name("copyfie-store".to_string())
            .spawn(move || {
                for job in rx {
                    job(&db);
                }
                // Dropping `db` here checkpoints/rolls back WAL cleanly.
            })
            .expect("failed to spawn the store actor thread");

        Self { tx }
    }

    /// Run `op` on the actor thread and await its result.
    pub async fn call<T, F>(&self, op: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&Database) -> std::result::Result<T, StoreError> + Send + 'static,
    {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.tx
            .send(Box::new(move |db| {
                // The caller may have gone away; ignore the send result.
                let _ = reply_tx.send(op(db));
            }))
            .map_err(|_| ActorError::Stopped)?;

        match reply_rx.await {
            Ok(result) => result.map_err(ActorError::from),
            Err(_) => Err(ActorError::Stopped),
        }
    }

    /// Blocking variant for dedicated threads (capture, cleanup, blob GC).
    ///
    /// Never call this from the async runtime or the UI thread: it waits on
    /// the actor thread to finish the operation.
    pub fn call_blocking<T, F>(&self, op: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&Database) -> std::result::Result<T, StoreError> + Send + 'static,
    {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);

        self.tx
            .send(Box::new(move |db| {
                let _ = reply_tx.send(op(db));
            }))
            .map_err(|_| ActorError::Stopped)?;

        match reply_rx.recv() {
            Ok(result) => result.map_err(ActorError::from),
            Err(_) => Err(ActorError::Stopped),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use copyfie_core::{NewItem, Retention, SourceApp};

    fn text(value: &str, at: i64) -> NewItem {
        NewItem::text(value, SourceApp::default(), at, Retention::Days(30))
    }

    #[tokio::test]
    async fn runs_operations_on_the_actor() {
        let handle = StoreHandle::spawn(Database::open_in_memory().expect("db"));

        let inserted = handle
            .call(|db| db.upsert(&text("actor", 1)))
            .await
            .expect("upsert");
        assert!(inserted.is_new);

        let count = handle.call(|db| db.count(None)).await.expect("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn serializes_concurrent_writers() {
        let handle = StoreHandle::spawn(Database::open_in_memory().expect("db"));

        let mut tasks = Vec::new();
        for index in 0..50 {
            let handle = handle.clone();
            tasks.push(tokio::spawn(async move {
                handle
                    .call(move |db| db.upsert(&text(&format!("item {index}"), index as i64)))
                    .await
            }));
        }

        for task in tasks {
            assert!(task.await.expect("join").expect("call").is_new);
        }

        let count = handle.call(|db| db.count(None)).await.expect("count");
        assert_eq!(count, 50);
    }

    #[tokio::test]
    async fn reports_store_errors_without_killing_the_actor() {
        let handle = StoreHandle::spawn(Database::open_in_memory().expect("db"));

        // Request a row that cannot exist; then prove the actor still works.
        let missing = handle
            .call(|db| db.set_pinned(42, true, 0, Retention::Days(30)))
            .await
            .expect("pinned");
        assert!(!missing);

        let count = handle.call(|db| db.count(None)).await.expect("count");
        assert_eq!(count, 0);
    }
}
