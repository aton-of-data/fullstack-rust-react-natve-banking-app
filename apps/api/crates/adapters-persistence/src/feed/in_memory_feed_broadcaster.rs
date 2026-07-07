//! In-process feed broadcaster with optional PostgreSQL LISTEN/NOTIFY bridge.

use std::sync::Arc;

use async_trait::async_trait;
use ficus_application::ports::{FeedBroadcaster, FeedItem};
use ficus_domain::errors::DomainError;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::mapper::FEED_NOTIFY_CHANNEL;

/// Broadcasts feed items in-process and optionally via PostgreSQL NOTIFY.
pub struct InMemoryFeedBroadcaster {
    sender: broadcast::Sender<FeedItem>,
    db: Option<DatabaseConnection>,
    listener_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
}

impl InMemoryFeedBroadcaster {
    /// Creates an in-memory broadcaster with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(16));
        Self {
            sender,
            db: None,
            listener_handle: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Attaches a PostgreSQL connection for NOTIFY publishing and LISTEN forwarding.
    pub fn with_pg_notify(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    /// Spawns a background task that LISTENs on the feed channel and forwards events.
    pub async fn spawn_pg_listener(&self) -> Result<(), DomainError> {
        let Some(db) = self.db.clone() else {
            return Ok(());
        };

        let mut guard = self.listener_handle.lock().await;
        if guard.is_some() {
            return Ok(());
        }

        let sender = self.sender.clone();
        let handle = tokio::spawn(async move {
            if let Err(err) = run_pg_listener(db, sender).await {
                error!(error = %err, "feed pg listener terminated");
            }
        });
        *guard = Some(handle);
        info!("feed pg listener started");
        Ok(())
    }
}

#[async_trait]
impl FeedBroadcaster for InMemoryFeedBroadcaster {
    async fn publish(&self, item: FeedItem) -> Result<(), DomainError> {
        if let Some(db) = &self.db {
            if let Err(err) = notify_pg(db, &item).await {
                warn!(error = %err, "failed to NOTIFY feed event");
            }
        }

        let _ = self.sender.send(item);
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<FeedItem> {
        self.sender.subscribe()
    }
}

async fn notify_pg(db: &DatabaseConnection, item: &FeedItem) -> Result<(), DomainError> {
    let payload = serde_json::to_string(item)
        .map_err(|_| DomainError::Validation("feed serialization failed".into()))?;
    let escaped = payload.replace('\'', "''");
    let sql = format!("NOTIFY {FEED_NOTIFY_CHANNEL}, '{escaped}'");
    db.execute(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await
        .map_err(|err| DomainError::Validation(format!("notify failed: {err}")))?;
    Ok(())
}

async fn run_pg_listener(
    _db: DatabaseConnection,
    sender: broadcast::Sender<FeedItem>,
) -> Result<(), DomainError> {
    use futures::StreamExt;

    let listener_url = std::env::var("DATABASE_URL").map_err(|_| {
        DomainError::Validation("DATABASE_URL required for feed pg listener".into())
    })?;

    let mut listener = sqlx::postgres::PgListener::connect(&listener_url)
        .await
        .map_err(|err| DomainError::Validation(format!("pg listener connect failed: {err}")))?;

    listener
        .listen(FEED_NOTIFY_CHANNEL)
        .await
        .map_err(|err| DomainError::Validation(format!("pg listen failed: {err}")))?;

    let mut notifications = listener.into_stream();
    while let Some(notification) = notifications.next().await {
        match notification {
            Ok(note) => {
                if let Ok(item) = serde_json::from_str::<FeedItem>(note.payload()) {
                    let _ = sender.send(item);
                }
            }
            Err(err) => {
                warn!(error = %err, "feed pg notification error");
            }
        }
    }

    Ok(())
}
