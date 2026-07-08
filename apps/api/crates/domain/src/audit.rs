//! Append-only audit event drafts for security and financial operations.
//!
//! This module belongs to the domain layer. It defines event types and an
//! in-memory draft shape. Persistence adapters write these to `audit_events`.
//! Audit metadata must not include passwords, JWTs, or full card/PII payloads.
//! Prefer hashed idempotency keys when correlating retries in logs; drafts may
//! carry business fields needed for forensic reconstruction.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Audit event types for the append-only audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditEventType {
    /// Successful password authentication.
    LoginSuccess,
    /// Failed authentication attempt (unknown user or bad password).
    LoginFailure,
    /// Transfer accepted into transactional execution (pre-commit marker).
    TransferRequested,
    /// Transfer committed with ledger and balance updates.
    TransferCompleted,
    /// Transfer declined (e.g. insufficient funds) without committing money movement.
    TransferDeclined,
    /// Idempotent replay returned a stored response.
    IdempotencyReplay,
    /// Idempotency key reused with a conflicting fingerprint.
    IdempotencyConflict,
    /// Authorization failed at a protected boundary.
    AuthorizationFailure,
    /// Unexpected financial operation failure for ops triage.
    FinancialOperationFailure,
}

impl AuditEventType {
    /// Stable snake_case string persisted in `audit_events.event_type`.
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEventType::LoginSuccess => "login_success",
            AuditEventType::LoginFailure => "login_failure",
            AuditEventType::TransferRequested => "transfer_requested",
            AuditEventType::TransferCompleted => "transfer_completed",
            AuditEventType::TransferDeclined => "transfer_declined",
            AuditEventType::IdempotencyReplay => "idempotency_replay",
            AuditEventType::IdempotencyConflict => "idempotency_conflict",
            AuditEventType::AuthorizationFailure => "authorization_failure",
            AuditEventType::FinancialOperationFailure => "financial_operation_failure",
        }
    }
}

/// Draft audit event before persistence.
///
/// `request_id` and `trace_id` correlate HTTP and distributed traces.
/// `metadata` is a flat string map serialized to JSON by the persistence
/// mapper — keep values non-sensitive.
#[derive(Debug, Clone)]
pub struct AuditEventDraft {
    /// Event classification.
    pub event_type: AuditEventType,
    /// Acting user when known (absent for anonymous login failures).
    pub actor_user_id: Option<Uuid>,
    /// Related transfer when applicable.
    pub transfer_id: Option<Uuid>,
    /// HTTP / edge request correlation id.
    pub request_id: String,
    /// Distributed trace id.
    pub trace_id: String,
    /// Non-sensitive structured details.
    pub metadata: std::collections::BTreeMap<String, String>,
    /// Event timestamp (UTC).
    pub occurred_at: DateTime<Utc>,
}
