use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Audit event types for append-only audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailure,
    TransferRequested,
    TransferCompleted,
    TransferDeclined,
    IdempotencyReplay,
    IdempotencyConflict,
    AuthorizationFailure,
    FinancialOperationFailure,
}

impl AuditEventType {
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
#[derive(Debug, Clone)]
pub struct AuditEventDraft {
    pub event_type: AuditEventType,
    pub actor_user_id: Option<Uuid>,
    pub transfer_id: Option<Uuid>,
    pub request_id: String,
    pub trace_id: String,
    pub metadata: std::collections::BTreeMap<String, String>,
    pub occurred_at: DateTime<Utc>,
}
