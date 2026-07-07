use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ports::{BalanceRecord, FeedItem, LedgerEntryRecord, TransferRecord};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserSearchItem {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BalanceResponse {
    pub balance_minor: String,
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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LedgerItemResponse {
    pub entry_id: Uuid,
    pub transfer_id: Uuid,
    pub amount_minor: String,
    pub direction: String,
    pub currency: String,
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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TransferRequest {
    pub recipient_username: String,
    pub amount_minor: String,
    pub currency: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TransferResponse {
    pub transfer_id: Uuid,
    pub status: String,
    pub sender_balance_minor: String,
    pub currency: String,
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct FeedItemResponse {
    pub transfer_id: Uuid,
    pub sender_username: String,
    pub recipient_username: String,
    pub amount_minor: String,
    pub currency: String,
    pub description: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}
