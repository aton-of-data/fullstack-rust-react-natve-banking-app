//! Transfer and public-feed orchestration use cases.
//!
//! This module is the application-layer entry point for money movement and for
//! reading/subscribing to the public transfer feed. It coordinates domain
//! validation helpers and port traits; it does **not** open database
//! transactions, take row locks, or write ledger lines.
//!
//! # Layer responsibility
//!
//! [`TransferService`] owns the **retry-safe transfer use case**:
//!
//! 1. Validate the client idempotency key format (domain).
//! 2. Compute a request fingerprint over the logical payload (domain).
//! 3. Pre-check the idempotency store via [`crate::ports::IdempotencyRepository`].
//! 4. On miss, delegate transactional money movement to
//!    [`crate::ports::TransferExecutor`].
//! 5. After a **completed** transfer returns, best-effort publish a feed item.
//!
//! [`FeedService`] owns read-side feed listing and realtime subscription. Both
//! services are defined here; [`crate::feed`] re-exports [`FeedService`] for a
//! stable public path.
//!
//! # Idempotency model (application view)
//!
//! | Outcome | Condition | Behavior |
//! | ------- | --------- | -------- |
//! | Replay | Same sender + key + fingerprint | Return deserialized stored response; no executor call |
//! | Conflict | Same sender + key, different fingerprint | [`DomainError::IdempotencyConflict`]; no money movement |
//! | Execute | No stored record | Call [`TransferExecutor::execute_transfer`] |
//!
//! The executor is still responsible for durable idempotency storage, advisory
//! locks, and in-transaction conflict detection. This service's pre-check is an
//! optimistic fast path for retries, not a substitute for persistence locking.
//!
//! # Feed publication
//!
//! Feed items are published **only after** the executor returns a
//! [`TransferStatus::Completed`](ficus_domain::transfer::TransferStatus::Completed)
//! record. Publish failures are logged and **must not** roll back balances or
//! ledger entries — money movement has already committed.
//!
//! # What this module deliberately does not do
//!
//! - Acquire `FOR UPDATE` / advisory locks
//! - Write double-entry ledger rows or mutate account balances
//! - Choose SQL isolation levels
//! - Map domain errors to HTTP status codes

use std::sync::Arc;

use ficus_domain::errors::DomainError;
use ficus_domain::idempotency::{request_fingerprint, validate_idempotency_key};
use metrics::counter;
use tracing::warn;

use crate::ports::{
    FeedBroadcaster, FeedItem, IdempotencyRepository, Page, TransferExecutor, TransferRecord,
    TransferRepository,
};

/// Orchestrates retry-safe peer-to-peer transfers.
///
/// Wires an idempotency pre-check, a transactional [`TransferExecutor`], and an
/// optional post-success [`FeedBroadcaster`]. Constructed at the composition
/// root with `Arc<dyn …>` adapters so this crate stays free of SQL and crypto.
pub struct TransferService {
    executor: Arc<dyn TransferExecutor>,
    idempotency: Arc<dyn IdempotencyRepository>,
    feed: Arc<dyn FeedBroadcaster>,
}

impl TransferService {
    /// Creates a transfer service over the given ports.
    ///
    /// # Arguments
    ///
    /// * `executor` — transactional money movement (locks, ledger, balances, idempotency insert)
    /// * `idempotency` — read path for stored responses / conflict detection before execute
    /// * `feed` — post-commit public feed publisher (best-effort)
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

    /// Executes a retry-safe transfer for `sender_user_id`.
    ///
    /// # Flow
    ///
    /// 1. Reject malformed idempotency keys with
    ///    [`DomainError::InvalidIdempotencyKey`].
    /// 2. Fingerprint `(sender, recipient, amount_minor, currency, description)`.
    /// 3. If an idempotency record exists for `(sender, key)`:
    ///    - **Different fingerprint** → [`DomainError::IdempotencyConflict`]
    ///      (metrics: `ficus_transfer_idempotency_conflict_total`).
    ///    - **Same fingerprint** → deserialize and return the stored
    ///      [`TransferRecord`] (metrics: `ficus_transfer_idempotency_replay_total`);
    ///      the executor is not invoked.
    /// 4. Otherwise parse `amount_minor` as `i64` and call
    ///    [`TransferExecutor::execute_transfer`].
    /// 5. If the returned status is `Completed`, publish a [`FeedItem`]. Feed
    ///    errors are warned and swallowed so funds are never rolled back.
    ///
    /// # Parameters
    ///
    /// * `amount_minor` — decimal string of integer minor units (e.g. `"1500"` for $15.00 USD)
    /// * `idempotency_key` — client retry token scoped to the sender
    /// * `request_id` / `trace_id` — correlation identifiers forwarded to the executor/audit path
    ///
    /// # Errors
    ///
    /// Propagates domain/port errors from validation, idempotency lookup,
    /// amount parse, and executor execution. Corrupt stored JSON becomes
    /// [`DomainError::Validation`].
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
                counter!("ficus_transfer_idempotency_conflict_total").increment(1);
                return Err(DomainError::IdempotencyConflict);
            }
            counter!("ficus_transfer_idempotency_replay_total").increment(1);
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
            if let Err(err) = self.feed.publish(item).await {
                warn!(transfer_id = %record.id, error = %err, "failed to publish feed event after transfer");
            }
        }

        Ok(record)
    }
}

use uuid::Uuid;

/// Read and subscribe use cases for the public transfer feed.
///
/// Listing is served from [`TransferRepository`]; live updates come from
/// [`FeedBroadcaster::subscribe`]. This type lives in the transfer module
/// because feed items are projections of completed transfers; see [`crate::feed`]
/// for the re-export used by the crate root.
pub struct FeedService {
    transfers: Arc<dyn TransferRepository>,
    feed: Arc<dyn FeedBroadcaster>,
}

impl FeedService {
    /// Creates a feed service over transfer read and broadcast ports.
    pub fn new(transfers: Arc<dyn TransferRepository>, feed: Arc<dyn FeedBroadcaster>) -> Self {
        Self { transfers, feed }
    }

    /// Returns a cursor-paginated page of public feed items.
    ///
    /// Ordering and cursor encoding are defined by the
    /// [`TransferRepository::list_feed`] implementor. This method does not
    /// filter by authenticated user — the feed is intentionally public.
    pub async fn list(
        &self,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<FeedItem>, DomainError> {
        self.transfers.list_feed(cursor, limit).await
    }

    /// Subscribes to realtime feed publications from the broadcaster.
    ///
    /// Returned receiver semantics (lag, capacity) belong to the
    /// [`FeedBroadcaster`] implementor. Missed events after lag must be
    /// recovered via [`Self::list`].
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<FeedItem> {
        self.feed.subscribe()
    }
}
