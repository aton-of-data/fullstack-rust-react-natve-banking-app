use std::sync::Arc;

use ficus_domain::errors::DomainError;
use ficus_domain::idempotency::{request_fingerprint, validate_idempotency_key};

use crate::ports::{
    FeedBroadcaster, FeedItem, IdempotencyRepository, Page, TransferExecutor, TransferRecord,
    TransferRepository,
};

/// Transfer orchestration use cases.
pub struct TransferService {
    executor: Arc<dyn TransferExecutor>,
    idempotency: Arc<dyn IdempotencyRepository>,
    feed: Arc<dyn FeedBroadcaster>,
}

impl TransferService {
    pub fn new(
        executor: Arc<dyn TransferExecutor>,
        idempotency: Arc<dyn IdempotencyRepository>,
        feed: Arc<dyn FeedBroadcaster>,
    ) -> Self {
        Self {
            executor,
            idempotency,
            feed,
        }
    }

    /// Executes a retry-safe transfer.
    #[allow(clippy::too_many_arguments)]
    pub async fn transfer(
        &self,
        sender_user_id: Uuid,
        recipient_username: &str,
        amount_minor: &str,
        currency: &str,
        description: Option<&str>,
        idempotency_key: &str,
        request_id: &str,
        trace_id: &str,
    ) -> Result<TransferRecord, DomainError> {
        if !validate_idempotency_key(idempotency_key) {
            return Err(DomainError::InvalidIdempotencyKey);
        }

        let fingerprint = request_fingerprint(
            &sender_user_id.to_string(),
            recipient_username,
            amount_minor,
            currency,
            description.unwrap_or(""),
        );

        if let Some(existing) = self
            .idempotency
            .find(sender_user_id, idempotency_key)
            .await?
        {
            if existing.fingerprint != fingerprint {
                return Err(DomainError::IdempotencyConflict);
            }
            return serde_json::from_str(&existing.response_body)
                .map_err(|_| DomainError::Validation("stored response corrupt".into()));
        }

        let record = self
            .executor
            .execute_transfer(
                sender_user_id,
                recipient_username,
                amount_minor
                    .parse::<i64>()
                    .map_err(|_| DomainError::InvalidMoney("invalid amount".into()))?,
                currency,
                description,
                idempotency_key,
                &fingerprint,
                request_id,
                trace_id,
            )
            .await?;

        if record.status == ficus_domain::transfer::TransferStatus::Completed {
            let item = FeedItem {
                transfer_id: record.id,
                sender_username: record.sender_username.clone(),
                recipient_username: record.recipient_username.clone(),
                amount_minor: record.amount_minor.to_string(),
                currency: record.currency_code.clone(),
                description: record.description.clone(),
                created_at: record.created_at,
            };
            let _ = self.feed.publish(item).await;
        }

        Ok(record)
    }
}

use uuid::Uuid;

/// Feed query use cases.
pub struct FeedService {
    transfers: Arc<dyn TransferRepository>,
    feed: Arc<dyn FeedBroadcaster>,
}

impl FeedService {
    pub fn new(transfers: Arc<dyn TransferRepository>, feed: Arc<dyn FeedBroadcaster>) -> Self {
        Self { transfers, feed }
    }

    pub async fn list(
        &self,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<FeedItem>, DomainError> {
        self.transfers.list_feed(cursor, limit).await
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<FeedItem> {
        self.feed.subscribe()
    }
}
