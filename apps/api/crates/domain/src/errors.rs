//! Transport-agnostic domain errors for money movement and auth.
//!
//! This module belongs to the domain layer. Errors here describe business and
//! validation failures without HTTP status codes. The HTTP adapter maps these
//! variants to stable client-facing statuses and error bodies.
//!
//! # Mapping guidance
//!
//! | Variant | Typical client meaning | Retryable? |
//! | ------- | ---------------------- | ---------- |
//! | Validation / InvalidMoney / InvalidIdempotencyKey | 400 | No |
//! | InvalidCredentials | 401 (generic) | No |
//! | IdempotencyConflict | 409 | No (fix payload) |
//! | InsufficientFunds / SelfTransfer / RecipientNotFound | 422 | No |
//! | UnbalancedLedger / NegativeBalance | 500 (should not reach client in healthy code) | No |

use thiserror::Error;

/// Domain-level errors — transport-agnostic.
///
/// Variants are safe to expose as **codes** via HTTP mapping. Detail strings
/// must not contain passwords, tokens, raw idempotency keys, or internal SQL.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    /// Money parsing or arithmetic failed (negative, fractional, overflow).
    ///
    /// User-correctable validation error. Map to HTTP 400. Do not audit as a
    /// completed financial mutation.
    #[error("invalid money amount: {0}")]
    InvalidMoney(String),

    /// Currency code is not in the supported set, or currencies mismatched.
    ///
    /// User-correctable or programming error at the boundary. Map to HTTP 400.
    #[error("unsupported currency: {0}")]
    UnsupportedCurrency(String),

    /// Sender available balance is below the requested transfer amount.
    ///
    /// User-correctable business error. Must not create transfer or ledger rows.
    /// Persistence rolls back the transaction; declined-transfer audit may be
    /// recorded outside the aborted transaction. Map to HTTP 422.
    #[error("insufficient funds")]
    InsufficientFunds,

    /// Sender and recipient resolve to the same user or account.
    ///
    /// User-correctable. Map to HTTP 422. No ledger writes.
    #[error("cannot transfer to self")]
    SelfTransfer,

    /// Recipient username does not exist.
    ///
    /// User-correctable. Map to HTTP 422. Avoid user-enumeration beyond what
    /// product policy already allows for search.
    #[error("recipient not found")]
    RecipientNotFound,

    /// User id was not found for an authenticated or looked-up identity.
    ///
    /// Often indicates stale token or data inconsistency. Map carefully (401
    /// vs 404) at the HTTP edge.
    #[error("user not found")]
    UserNotFound,

    /// Login failed (unknown user or bad password).
    ///
    /// Always return the same generic message to clients. Audit login failure
    /// without storing the password. Map to HTTP 401.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Idempotency key reused with a different request fingerprint.
    ///
    /// Conflict — clients must not treat this as a successful retry. Map to
    /// HTTP 409. Audit as idempotency conflict.
    #[error("idempotency conflict")]
    IdempotencyConflict,

    /// Idempotency key missing or failing [`crate::idempotency::validate_idempotency_key`].
    ///
    /// User-correctable validation. Map to HTTP 400.
    #[error("invalid idempotency key")]
    InvalidIdempotencyKey,

    /// Referenced transfer does not exist.
    ///
    /// Map to HTTP 404 when exposed.
    #[error("transfer not found")]
    TransferNotFound,

    /// Account missing for a user (data integrity issue for funded users).
    ///
    /// Map to HTTP 404/500 depending on operation.
    #[error("account not found")]
    AccountNotFound,

    /// Ledger entry set does not sum to zero under signed-amount semantics.
    ///
    /// Programming / integrity error — must not commit. Prefer internal 500
    /// mapping; should be impossible for standard P2P pairs from
    /// [`crate::ledger::build_transfer_entries`].
    #[error("ledger entries do not balance")]
    UnbalancedLedger,

    /// Operation would leave a projected balance negative.
    ///
    /// Enforced in domain checks and DB constraints. Treat like insufficient
    /// funds / integrity failure — roll back; do not expose internal maths.
    #[error("negative balance not allowed")]
    NegativeBalance,

    /// Generic validation or mapped transient DB messaging (e.g. serialization).
    ///
    /// When the message indicates serialization/deadlock, the transfer executor
    /// may retry. Otherwise treat as client or internal validation failure.
    #[error("validation error: {0}")]
    Validation(String),
}
