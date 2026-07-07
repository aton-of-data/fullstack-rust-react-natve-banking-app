use thiserror::Error;

/// Domain-level errors — transport-agnostic.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid money amount: {0}")]
    InvalidMoney(String),

    #[error("unsupported currency: {0}")]
    UnsupportedCurrency(String),

    #[error("insufficient funds")]
    InsufficientFunds,

    #[error("cannot transfer to self")]
    SelfTransfer,

    #[error("recipient not found")]
    RecipientNotFound,

    #[error("user not found")]
    UserNotFound,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("idempotency conflict")]
    IdempotencyConflict,

    #[error("invalid idempotency key")]
    InvalidIdempotencyKey,

    #[error("transfer not found")]
    TransferNotFound,

    #[error("account not found")]
    AccountNotFound,

    #[error("ledger entries do not balance")]
    UnbalancedLedger,

    #[error("negative balance not allowed")]
    NegativeBalance,

    #[error("validation error: {0}")]
    Validation(String),
}
