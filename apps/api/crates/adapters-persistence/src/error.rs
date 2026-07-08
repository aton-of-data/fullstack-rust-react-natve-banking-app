//! Persistence error mapping helpers.
//!
//! Translates SeaORM [`DbErr`] values into [`DomainError`] at repository and
//! executor boundaries so application/domain layers never observe ORM types.
//!
//! # Retryable detection
//!
//! [`is_retryable_db_error`] inspects the lowercase message of query/exec/conn
//! errors for:
//!
//! - `"could not serialize access"`
//! - `"deadlock detected"`
//! - `"serialization failure"`
//!
//! Under **ReadCommitted** transfers, these typically come from deadlocks or
//! rare conflict wording rather than true `SERIALIZABLE` aborts. The transfer
//! executor remaps matching errors to `Validation("serialization failure: …")`
//! so its retry loop can recognize them.
//!
//! # Constraint mapping
//!
//! [`map_balance_constraint`] detects the `chk_account_balances_non_negative`
//! check violation and returns [`DomainError::NegativeBalance`]. Other DB
//! failures fall through to the generic [`map_db_err`] path.

use ficus_domain::errors::DomainError;
use sea_orm::DbErr;

/// Returns true when the database error is retryable (serialization wording / deadlock).
///
/// Used by the transfer executor before and during transactions. Message matching
/// is intentionally broad; isolation remains ReadCommitted (see executor docs).
pub fn is_retryable_db_error(err: &DbErr) -> bool {
    match err {
        DbErr::Query(runtime) => {
            let msg = runtime.to_string().to_lowercase();
            msg.contains("could not serialize access")
                || msg.contains("deadlock detected")
                || msg.contains("serialization failure")
        }
        DbErr::Exec(runtime) => {
            let msg = runtime.to_string().to_lowercase();
            msg.contains("could not serialize access")
                || msg.contains("deadlock detected")
                || msg.contains("serialization failure")
        }
        DbErr::Conn(runtime) => {
            let msg = runtime.to_string().to_lowercase();
            msg.contains("could not serialize access") || msg.contains("deadlock detected")
        }
        _ => false,
    }
}

/// Maps SeaORM errors to domain errors for repository boundaries.
///
/// Logs unexpected failures and returns a generic validation message so raw
/// database details are not leaked to API clients. `RecordNotFound` maps to a
/// validation "record not found" (callers that need typed not-found errors
/// check for `None` before calling this).
pub fn map_db_err(err: DbErr) -> DomainError {
    if let DbErr::RecordNotFound(_) = err {
        return DomainError::Validation("record not found".into());
    }

    tracing::error!(error = %err, "database error");
    DomainError::Validation("database operation failed".into())
}

/// Maps known balance check-constraint violations to domain errors.
///
/// Returns [`Some`]`(`[`DomainError::NegativeBalance`]`)` when the error text
/// mentions `chk_account_balances_non_negative`; otherwise [`None`] so callers
/// can fall back to [`map_db_err`] / txn mapping.
pub fn map_balance_constraint(err: &DbErr) -> Option<DomainError> {
    let msg = err.to_string().to_lowercase();
    if msg.contains("chk_account_balances_non_negative") {
        return Some(DomainError::NegativeBalance);
    }
    None
}
