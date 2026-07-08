//! Repository and infrastructure ports for the application layer.
//!
//! Ports are the hexagonal boundaries between use cases and adapters. Services
//! in this crate depend only on these traits and record DTOs — never on Axum
//! types, SeaORM entity modules, SQL strings, or crypto libraries.
//!
//! # Anti-corruption rule
//!
//! **SeaORM entities and persistence models must not leak through these ports.**
//! Implementors map storage rows into the `*Record` / [`FeedItem`] types defined
//! here (or into domain types such as [`AuditEventDraft`]) before returning.
//! Callers must be able to treat this module as the entire persistence surface.
//!
//! # Implementor obligations (shared)
//!
//! - Map storage failures into [`DomainError`] (or a mapped domain variant);
//!   do not panic on expected not-found / conflict paths.
//! - Remain `Send + Sync` for sharing behind `Arc<dyn …>` at the composition root.
//! - Never return ORM entity structs, `DatabaseConnection`, or SQL error types
//!   across the trait boundary.
//! - Treat secrets (`password_hash`, tokens) as sensitive: do not log them.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use ficus_domain::audit::AuditEventDraft;
use ficus_domain::errors::DomainError;
use ficus_domain::transfer::TransferStatus;

use serde::{Deserialize, Serialize};

/// Application-facing user projection loaded from persistence.
///
/// Contains the password hash only so authentication can verify credentials via
/// [`PasswordHasher`]. HTTP adapters must never serialize this type to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    /// Stable user identifier.
    pub id: Uuid,
    /// Unique public username used for login and transfer addressing.
    pub username: String,
    /// Stored password hash (Argon2 or equivalent). Never log or return over HTTP.
    pub password_hash: String,
}

/// Account balance projection for a single currency wallet.
#[derive(Debug, Clone)]
pub struct BalanceRecord {
    /// Account that owns this balance row.
    pub account_id: Uuid,
    /// Available balance in integer minor units (e.g. cents).
    pub balance_minor: i64,
    /// ISO-like currency code associated with the account (e.g. `"USD"`).
    pub currency_code: String,
}

/// Persisted transfer result returned to application services and idempotency replay.
///
/// Serializable so stored idempotency response bodies can round-trip through
/// [`TransferService`](crate::TransferService) replay without re-executing money movement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    /// Transfer identifier.
    pub id: Uuid,
    /// Debited account id.
    pub sender_account_id: Uuid,
    /// Credited account id.
    pub recipient_account_id: Uuid,
    /// Sending user id (auth subject).
    pub sender_user_id: Uuid,
    /// Receiving user id.
    pub recipient_user_id: Uuid,
    /// Sender username at transfer time (denormalized for feed/UI).
    pub sender_username: String,
    /// Recipient username at transfer time.
    pub recipient_username: String,
    /// Transferred amount in integer minor units.
    pub amount_minor: i64,
    /// Currency code of the movement.
    pub currency_code: String,
    /// Optional human-readable memo.
    pub description: Option<String>,
    /// Domain status (`Completed` or `Declined`).
    pub status: TransferStatus,
    /// Creation timestamp (UTC).
    pub created_at: DateTime<Utc>,
}

/// Stored idempotency outcome for a `(sender_user_id, idempotency_key)` pair.
///
/// Used by the transfer pre-check to distinguish safe replays from conflicts.
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    /// SHA-256 hex fingerprint of the protected request payload.
    pub fingerprint: String,
    /// Serialized prior response body (typically JSON of [`TransferRecord`]).
    pub response_body: String,
    /// HTTP-oriented status code stored with the response for adapter replay.
    pub status_code: u16,
}

/// Single double-entry ledger line projected for API/history use.
#[derive(Debug, Clone)]
pub struct LedgerEntryRecord {
    /// Ledger entry id.
    pub id: Uuid,
    /// Account this line belongs to.
    pub account_id: Uuid,
    /// Related transfer id.
    pub transfer_id: Uuid,
    /// Absolute amount in minor units (sign/direction carried by [`Self::direction`]).
    pub amount_minor: i64,
    /// Movement direction relative to the account (e.g. `"debit"` / `"credit"`).
    pub direction: String,
    /// Currency of the entry.
    pub currency_code: String,
    /// When the entry was recorded (UTC).
    pub created_at: DateTime<Utc>,
}

/// Public feed item describing a completed transfer.
///
/// Amounts are strings of integer minor units for wire compatibility with HTTP DTOs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedItem {
    /// Completed transfer id.
    pub transfer_id: Uuid,
    /// Sender's public username.
    pub sender_username: String,
    /// Recipient's public username.
    pub recipient_username: String,
    /// Amount as a string of integer minor units (e.g. `"1500"`).
    pub amount_minor: String,
    /// Currency code.
    pub currency: String,
    /// Optional transfer description/memo.
    pub description: Option<String>,
    /// Transfer creation time (UTC).
    pub created_at: DateTime<Utc>,
}

/// Cursor-paginated page of `T`.
///
/// Cursor opacity and sort order are defined by each repository implementor;
/// clients must treat `next_cursor` as an opaque token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Page<T> {
    /// Page contents in repository-defined order.
    pub items: Vec<T>,
    /// Opaque cursor for the next page, or `None` when exhausted.
    pub next_cursor: Option<String>,
}

/// Port for loading and searching users.
///
/// Implementors must not expose ORM entities. [`UserRecord::password_hash`] is
/// required for login verification but must never be included in search HTTP
/// responses (strip at the DTO/mapper boundary).
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Finds a user by exact username, if present.
    async fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>, DomainError>;

    /// Finds a user by primary id, if present.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRecord>, DomainError>;

    /// Prefix/search by username, excluding `exclude_user_id` (typically the caller).
    ///
    /// Must honor `cursor` / `limit` pagination semantics consistently with
    /// other list endpoints. Empty or oversized queries are validated by the
    /// application service before calling this method.
    async fn search_by_username(
        &self,
        query: &str,
        exclude_user_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<UserRecord>, DomainError>;
}

/// Port for account lookup, balances, and ledger history.
///
/// Balance and ledger reads are projections; they must not perform transfers.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Resolves the primary account id for a user, if one exists.
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Uuid>, DomainError>;

    /// Returns the current balance projection for `account_id`.
    ///
    /// Implementors should fail with a domain-mapped not-found error when the
    /// account is missing rather than returning a zero synthetic balance.
    async fn get_balance(&self, account_id: Uuid) -> Result<BalanceRecord, DomainError>;

    /// Returns cursor-paginated ledger entries for `account_id` (newest-first or
    /// implementor-documented order).
    async fn get_ledger(
        &self,
        account_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<LedgerEntryRecord>, DomainError>;
}

/// Port for transfer lookups and public feed materialization from storage.
#[async_trait]
pub trait TransferRepository: Send + Sync {
    /// Loads a transfer by id, if present.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<TransferRecord>, DomainError>;

    /// Lists completed transfers as public [`FeedItem`]s with cursor pagination.
    async fn list_feed(
        &self,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<FeedItem>, DomainError>;
}

/// Port for idempotency record lookup and durable storage.
///
/// Keys are scoped to `sender_user_id`. Implementors that participate in the
/// transfer transaction must enforce uniqueness and conflict rules under the
/// same locking story as [`TransferExecutor`].
#[async_trait]
pub trait IdempotencyRepository: Send + Sync {
    /// Returns the stored record for `(sender_user_id, idempotency_key)`, if any.
    async fn find(
        &self,
        sender_user_id: Uuid,
        idempotency_key: &str,
    ) -> Result<Option<IdempotencyRecord>, DomainError>;

    /// Persists a successful (or intentionally stored) response for later replay.
    ///
    /// `fingerprint` must match the request used to produce `response_body`.
    /// Concurrent inserts for the same key with a different fingerprint must
    /// surface as an idempotency conflict at the executor/repository layer.
    async fn store(
        &self,
        sender_user_id: Uuid,
        idempotency_key: &str,
        fingerprint: &str,
        response_body: &str,
        status_code: u16,
    ) -> Result<(), DomainError>;
}

/// Port for appending security / money-movement audit events.
///
/// Append-only: implementors must not mutate prior events. Metadata must never
/// include passwords or raw secrets.
#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Appends a single audit event draft to durable storage.
    async fn append(&self, event: AuditEventDraft) -> Result<(), DomainError>;
}

/// Port for the transactional transfer execution path.
///
/// This is where adapters acquire locks, mutate balances, write ledger lines,
/// and store idempotency responses inside a database transaction. The
/// application service validates keys/fingerprints and publishes feed events
/// **outside** this boundary after success.
///
/// # Implementor guarantees
///
/// - Atomic money movement + ledger consistency for a single transfer attempt
/// - Durable idempotency outcome compatible with [`IdempotencyRepository`]
/// - No SeaORM entity types in the return value (map to [`TransferRecord`])
#[async_trait]
pub trait TransferExecutor: Send + Sync {
    /// Executes one transfer attempt under transactional isolation chosen by the adapter.
    ///
    /// `amount_minor` is already parsed integer minor units. `fingerprint` is
    /// the application-computed payload hash that must be stored/compared with
    /// the idempotency key.
    #[allow(clippy::too_many_arguments)]
    async fn execute_transfer(
        &self,
        sender_user_id: Uuid,
        recipient_username: &str,
        amount_minor: i64,
        currency_code: &str,
        description: Option<&str>,
        idempotency_key: &str,
        fingerprint: &str,
        request_id: &str,
        trace_id: &str,
    ) -> Result<TransferRecord, DomainError>;
}

/// Port for publishing and subscribing to public feed events.
///
/// Publish is best-effort relative to money movement: application code must
/// tolerate publish failure after a completed transfer without rolling back funds.
#[async_trait]
pub trait FeedBroadcaster: Send + Sync {
    /// Publishes a feed item to realtime subscribers (and any fan-out the adapter owns).
    async fn publish(&self, item: FeedItem) -> Result<(), DomainError>;

    /// Returns a new broadcast receiver for live feed items.
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<FeedItem>;
}

/// Port for password hashing and verification.
///
/// Implementors (e.g. Argon2) live in infrastructure. Passwords passed to these
/// methods must never be logged by callers or implementors.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hashes a plaintext password for storage.
    async fn hash(&self, password: &str) -> Result<String, DomainError>;

    /// Verifies a plaintext password against a stored hash.
    ///
    /// Returns `Ok(true)` on match, `Ok(false)` on mismatch. Timing-safe
    /// comparison is an implementor responsibility.
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError>;
}

/// Port for issuing and verifying bearer access tokens (typically JWT).
///
/// Token format and crypto are infrastructure concerns; this crate only needs
/// opaque strings and `(user_id, username)` claims.
#[async_trait]
pub trait TokenService: Send + Sync {
    /// Issues an access token for an authenticated user.
    async fn issue(&self, user_id: Uuid, username: &str) -> Result<String, DomainError>;

    /// Verifies a token and returns `(user_id, username)` claims on success.
    async fn verify(&self, token: &str) -> Result<(Uuid, String), DomainError>;
}
