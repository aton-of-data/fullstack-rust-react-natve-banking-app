use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use ficus_domain::audit::AuditEventDraft;
use ficus_domain::errors::DomainError;
use ficus_domain::transfer::TransferStatus;

use serde::{Deserialize, Serialize};

/// User record from persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

/// Account balance projection.
#[derive(Debug, Clone)]
pub struct BalanceRecord {
    pub account_id: Uuid,
    pub balance_minor: i64,
    pub currency_code: String,
}

/// Transfer result persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub id: Uuid,
    pub sender_account_id: Uuid,
    pub recipient_account_id: Uuid,
    pub sender_user_id: Uuid,
    pub recipient_user_id: Uuid,
    pub sender_username: String,
    pub recipient_username: String,
    pub amount_minor: i64,
    pub currency_code: String,
    pub description: Option<String>,
    pub status: TransferStatus,
    pub created_at: DateTime<Utc>,
}

/// Idempotency record.
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    pub fingerprint: String,
    pub response_body: String,
    pub status_code: u16,
}

/// Ledger entry record.
#[derive(Debug, Clone)]
pub struct LedgerEntryRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub transfer_id: Uuid,
    pub amount_minor: i64,
    pub direction: String,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
}

/// Feed item for public consumption.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedItem {
    pub transfer_id: Uuid,
    pub sender_username: String,
    pub recipient_username: String,
    pub amount_minor: String,
    pub currency: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Paginated result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRecord>, DomainError>;
    async fn search_by_username(
        &self,
        query: &str,
        exclude_user_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<UserRecord>, DomainError>;
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Uuid>, DomainError>;
    async fn get_balance(&self, account_id: Uuid) -> Result<BalanceRecord, DomainError>;
    async fn get_ledger(
        &self,
        account_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<LedgerEntryRecord>, DomainError>;
}

#[async_trait]
pub trait TransferRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<TransferRecord>, DomainError>;
    async fn list_feed(
        &self,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<FeedItem>, DomainError>;
}

#[async_trait]
pub trait IdempotencyRepository: Send + Sync {
    async fn find(
        &self,
        sender_user_id: Uuid,
        idempotency_key: &str,
    ) -> Result<Option<IdempotencyRecord>, DomainError>;
    async fn store(
        &self,
        sender_user_id: Uuid,
        idempotency_key: &str,
        fingerprint: &str,
        response_body: &str,
        status_code: u16,
    ) -> Result<(), DomainError>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn append(&self, event: AuditEventDraft) -> Result<(), DomainError>;
}

#[async_trait]
pub trait TransferExecutor: Send + Sync {
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

#[async_trait]
pub trait FeedBroadcaster: Send + Sync {
    async fn publish(&self, item: FeedItem) -> Result<(), DomainError>;
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<FeedItem>;
}

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &str) -> Result<String, DomainError>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError>;
}

#[async_trait]
pub trait TokenService: Send + Sync {
    async fn issue(&self, user_id: Uuid, username: &str) -> Result<String, DomainError>;
    async fn verify(&self, token: &str) -> Result<(Uuid, String), DomainError>;
}
