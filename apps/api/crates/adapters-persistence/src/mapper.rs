//! Maps between persistence models and application/domain types.
//!
//! # Boundary rule (ORM must not leak)
//!
//! SeaORM `entities::*::Model` values stay inside this crate. Public ports and
//! application use cases consume **records** (`UserRecord`, `TransferRecord`,
//! `FeedItem`, …) and domain types (`TransferStatus`, `AuditEventDraft`).
//! Mapping helpers here are the only supported translation layer.
//!
//! # Mapping rules
//!
//! - Timestamps: SeaORM `DateTimeWithTimeZone` → `chrono::DateTime<Utc>`.
//! - Transfer status: domain enum ↔ DB strings `"completed"` / `"declined"`.
//! - Feed cursor: `"{rfc3339}|{uuid}"` via [`encode_cursor`] / [`decode_cursor`].
//! - Amounts in feed items are stringified minor units for JSON clients.
//! - Audit metadata: domain `BTreeMap<String, String>` → JSON object for storage.
//! - Username joins for transfer/feed records happen in repositories; mappers
//!   only assemble the final DTO once usernames are known.

use chrono::{DateTime, Utc};
use ficus_application::ports::{
    BalanceRecord, FeedItem, IdempotencyRecord, LedgerEntryRecord, TransferRecord, UserRecord,
};
use ficus_domain::errors::DomainError;
use ficus_domain::transfer::TransferStatus;
use sea_orm::prelude::DateTimeWithTimeZone;

use crate::entities::{
    account_balances, audit_events, idempotency_requests, ledger_entries, transfers, users,
};

/// PostgreSQL channel used for feed NOTIFY events.
pub const FEED_NOTIFY_CHANNEL: &str = "ficus_feed_events";

/// Encodes a cursor from timestamp and identifier.
///
/// Format: `{created_at.to_rfc3339()}|{id}`. Used for transfer feed and ledger
/// pagination (newest-first scans).
pub fn encode_cursor(created_at: DateTime<Utc>, id: uuid::Uuid) -> String {
    format!("{}|{}", created_at.to_rfc3339(), id)
}

/// Decodes a pagination cursor into timestamp and identifier.
///
/// Rejects malformed strings, bad timestamps, or non-UUID ids with
/// [`DomainError::Validation`].
pub fn decode_cursor(cursor: &str) -> Result<(DateTime<Utc>, uuid::Uuid), DomainError> {
    let (ts, id) = cursor
        .split_once('|')
        .ok_or_else(|| DomainError::Validation("invalid cursor".into()))?;
    let created_at = DateTime::parse_from_rfc3339(ts)
        .map_err(|_| DomainError::Validation("invalid cursor timestamp".into()))?
        .with_timezone(&Utc);
    let id = uuid::Uuid::parse_str(id)
        .map_err(|_| DomainError::Validation("invalid cursor id".into()))?;
    Ok((created_at, id))
}

/// Converts a SeaORM timestamptz column to UTC.
fn to_utc(ts: DateTimeWithTimeZone) -> DateTime<Utc> {
    ts.with_timezone(&Utc)
}

/// Maps a transfer status enum to its database representation.
pub fn transfer_status_to_db(status: TransferStatus) -> &'static str {
    match status {
        TransferStatus::Completed => "completed",
        TransferStatus::Declined => "declined",
    }
}

/// Maps a database transfer status string to the domain enum.
///
/// Unknown values become [`DomainError::Validation`].
pub fn transfer_status_from_db(value: &str) -> Result<TransferStatus, DomainError> {
    match value {
        "completed" => Ok(TransferStatus::Completed),
        "declined" => Ok(TransferStatus::Declined),
        other => Err(DomainError::Validation(format!(
            "unknown transfer status: {other}"
        ))),
    }
}

/// Maps a user entity to an application record (including password hash).
pub fn user_to_record(model: users::Model) -> UserRecord {
    UserRecord {
        id: model.id,
        username: model.username,
        password_hash: model.password_hash,
    }
}

/// Maps an account balance entity to an application record.
pub fn balance_to_record(model: account_balances::Model) -> BalanceRecord {
    BalanceRecord {
        account_id: model.account_id,
        balance_minor: model.balance_minor,
        currency_code: model.currency_code,
    }
}

/// Maps a transfer entity and joined usernames to an application record.
pub fn transfer_to_record(
    model: transfers::Model,
    sender_username: String,
    recipient_username: String,
) -> Result<TransferRecord, DomainError> {
    Ok(TransferRecord {
        id: model.id,
        sender_account_id: model.sender_account_id,
        recipient_account_id: model.recipient_account_id,
        sender_user_id: model.sender_user_id,
        recipient_user_id: model.recipient_user_id,
        sender_username,
        recipient_username,
        amount_minor: model.amount_minor,
        currency_code: model.currency_code,
        description: model.description,
        status: transfer_status_from_db(&model.status)?,
        created_at: to_utc(model.created_at),
    })
}

/// Maps a transfer entity to a public feed item (usernames already resolved).
pub fn transfer_to_feed_item(
    model: transfers::Model,
    sender_username: String,
    recipient_username: String,
) -> FeedItem {
    FeedItem {
        transfer_id: model.id,
        sender_username,
        recipient_username,
        amount_minor: model.amount_minor.to_string(),
        currency: model.currency_code,
        description: model.description,
        created_at: to_utc(model.created_at),
    }
}

/// Maps a ledger entry entity to an application record.
pub fn ledger_entry_to_record(model: ledger_entries::Model) -> LedgerEntryRecord {
    LedgerEntryRecord {
        id: model.id,
        account_id: model.account_id,
        transfer_id: model.transfer_id,
        amount_minor: model.amount_minor,
        direction: model.direction,
        currency_code: model.currency_code,
        created_at: to_utc(model.created_at),
    }
}

/// Maps an idempotency entity to an application record.
pub fn idempotency_to_record(model: idempotency_requests::Model) -> IdempotencyRecord {
    IdempotencyRecord {
        fingerprint: model.fingerprint,
        response_body: model.response_body,
        status_code: model.status_code as u16,
    }
}

/// Maps domain audit metadata to JSON for persistence.
pub fn audit_metadata_to_json(
    metadata: &std::collections::BTreeMap<String, String>,
) -> serde_json::Value {
    serde_json::to_value(metadata).unwrap_or_else(|_| serde_json::json!({}))
}

/// Builds an audit event active model from a domain draft.
///
/// Assigns a new event UUID; ORM types never leave this function except as a
/// SeaORM insert target inside repositories/executor.
pub fn audit_draft_to_active(
    draft: ficus_domain::audit::AuditEventDraft,
) -> audit_events::ActiveModel {
    audit_events::ActiveModel {
        id: sea_orm::Set(uuid::Uuid::new_v4()),
        event_type: sea_orm::Set(draft.event_type.as_str().to_string()),
        actor_user_id: sea_orm::Set(draft.actor_user_id),
        transfer_id: sea_orm::Set(draft.transfer_id),
        request_id: sea_orm::Set(draft.request_id),
        trace_id: sea_orm::Set(draft.trace_id),
        metadata: sea_orm::Set(audit_metadata_to_json(&draft.metadata)),
        occurred_at: sea_orm::Set(draft.occurred_at.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_status_round_trip() {
        assert_eq!(
            transfer_status_from_db(transfer_status_to_db(TransferStatus::Completed)).unwrap(),
            TransferStatus::Completed
        );
        assert_eq!(
            transfer_status_from_db(transfer_status_to_db(TransferStatus::Declined)).unwrap(),
            TransferStatus::Declined
        );
    }

    #[test]
    fn cursor_round_trip() {
        let id = uuid::Uuid::new_v4();
        let ts = Utc::now();
        let cursor = encode_cursor(ts, id);
        let (decoded_ts, decoded_id) = decode_cursor(&cursor).unwrap();
        assert_eq!(decoded_id, id);
        assert_eq!(decoded_ts.timestamp(), ts.timestamp());
    }
}
