//! HTTP-oriented request/response DTOs shared by application services and adapters.
//!
//! These types are serde + utoipa schemas used at the application/HTTP boundary.
//! They intentionally avoid SeaORM entities and domain enums that should not
//! appear in OpenAPI (status values are mapped to stable strings).
//!
//! # Money encoding
//!
//! Wire amounts use **`amount_minor` / `balance_minor` as strings of integer
//! minor units** (for example `"1500"` for 15.00 USD). Integer strings avoid
//! floating-point JSON pitfalls and keep client parsing explicit.
//!
//! # Secrets
//!
//! [`LoginRequest::password`] must **never** be logged, traced, or written to
//! audit metadata. Prefer structured logging of usernames/request ids only.
//!
//! # Layer note
//!
//! Conversion `From` impls live here so HTTP handlers can map port records to
//! responses without re-encoding money rules. See [`TransferResponse`] for the
//! intentional `sender_balance_minor: "0"` placeholder filled by the HTTP layer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ports::{BalanceRecord, FeedItem, LedgerEntryRecord, TransferRecord};

/// Login credentials submitted by the client.
///
/// # Security
///
/// Never log [`Self::password`]. Authentication maps both unknown users and
/// bad passwords to a generic invalid-credentials error at the service layer.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    /// Account username.
    pub username: String,
    /// Plaintext password for verification only; never persist or log.
    pub password: String,
}

/// Successful login payload containing a bearer token and identity claims.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LoginResponse {
    /// Opaque access token (typically JWT) for subsequent authenticated calls.
    pub access_token: String,
    /// Authenticated user id.
    pub user_id: Uuid,
    /// Authenticated username.
    pub username: String,
}

/// Current-user profile returned by `GET /me`-style endpoints.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MeResponse {
    /// Authenticated user id.
    pub user_id: Uuid,
    /// Authenticated username.
    pub username: String,
}

/// One hit from username search (public identity only).
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserSearchItem {
    /// Matched user id.
    pub user_id: Uuid,
    /// Matched username.
    pub username: String,
}

/// Account balance response for the authenticated user.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BalanceResponse {
    /// Available balance as a string of integer minor units.
    pub balance_minor: String,
    /// Currency code (e.g. `"USD"`).
    pub currency: String,
}

impl From<BalanceRecord> for BalanceResponse {
    fn from(r: BalanceRecord) -> Self {
        Self {
            balance_minor: r.balance_minor.to_string(),
            currency: r.currency_code,
        }
    }
}

/// One ledger history line for the authenticated user's account.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LedgerItemResponse {
    /// Ledger entry id.
    pub entry_id: Uuid,
    /// Related transfer id.
    pub transfer_id: Uuid,
    /// Absolute amount as a string of integer minor units.
    pub amount_minor: String,
    /// Direction relative to the user's account (e.g. `"debit"` / `"credit"`).
    pub direction: String,
    /// Currency code.
    pub currency: String,
    /// Entry timestamp (UTC).
    pub created_at: DateTime<Utc>,
}

impl From<LedgerEntryRecord> for LedgerItemResponse {
    fn from(r: LedgerEntryRecord) -> Self {
        Self {
            entry_id: r.id,
            transfer_id: r.transfer_id,
            amount_minor: r.amount_minor.to_string(),
            direction: r.direction,
            currency: r.currency_code,
            created_at: r.created_at,
        }
    }
}

/// Client payload for creating a peer-to-peer transfer.
///
/// Sender identity comes from authentication context, not this body. Idempotency
/// is supplied via HTTP headers, not fields on this DTO.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TransferRequest {
    /// Recipient username (resolved by the executor/repository).
    pub recipient_username: String,
    /// Transfer amount as a string of integer minor units (e.g. `"1500"`).
    pub amount_minor: String,
    /// Currency code; must match supported account currency rules in domain/executor.
    pub currency: String,
    /// Optional memo shown in feed and history.
    pub description: Option<String>,
}

/// Transfer outcome returned to the client after execute or idempotent replay.
///
/// [`Self::sender_balance_minor`] is set to `"0"` by the [`From<TransferRecord>`]
/// impl because [`TransferRecord`] does not carry a post-transfer balance.
/// **The HTTP adapter must overwrite this field** with the sender's current
/// balance after the transfer use case returns.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TransferResponse {
    /// Created (or replayed) transfer id.
    pub transfer_id: Uuid,
    /// Stable status string: `"COMPLETED"` or `"DECLINED"`.
    pub status: String,
    /// Sender balance in minor units after the transfer; placeholder `"0"` until HTTP fills it.
    pub sender_balance_minor: String,
    /// Currency of the transfer / balance.
    pub currency: String,
    /// Transfer creation timestamp (UTC).
    pub created_at: DateTime<Utc>,
}

impl From<TransferRecord> for TransferResponse {
    fn from(r: TransferRecord) -> Self {
        Self {
            transfer_id: r.id,
            status: match r.status {
                ficus_domain::transfer::TransferStatus::Completed => "COMPLETED".into(),
                ficus_domain::transfer::TransferStatus::Declined => "DECLINED".into(),
            },
            sender_balance_minor: "0".into(),
            currency: r.currency_code,
            created_at: r.created_at,
        }
    }
}

/// Public feed item as exposed over HTTP / websocket payloads.
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct FeedItemResponse {
    /// Completed transfer id.
    pub transfer_id: Uuid,
    /// Sender username.
    pub sender_username: String,
    /// Recipient username.
    pub recipient_username: String,
    /// Amount as a string of integer minor units.
    pub amount_minor: String,
    /// Currency code.
    pub currency: String,
    /// Optional transfer description.
    pub description: Option<String>,
    /// Transfer creation timestamp (UTC).
    pub created_at: DateTime<Utc>,
}

impl From<FeedItem> for FeedItemResponse {
    fn from(item: FeedItem) -> Self {
        Self {
            transfer_id: item.transfer_id,
            sender_username: item.sender_username,
            recipient_username: item.recipient_username,
            amount_minor: item.amount_minor,
            currency: item.currency,
            description: item.description,
            created_at: item.created_at,
        }
    }
}

/// Generic cursor-paginated envelope for list endpoints.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PageResponse<T> {
    /// Page items.
    pub items: Vec<T>,
    /// Opaque next-page cursor, or `None` when no further pages exist.
    pub next_cursor: Option<String>,
}
