//! Persistence error mapping helpers.

use ficus_domain::errors::DomainError;
use sea_orm::DbErr;

/// Returns true when the database error is retryable (serialization/deadlock).
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
pub fn map_db_err(err: DbErr) -> DomainError {
    if let DbErr::RecordNotFound(_) = err {
        return DomainError::Validation("record not found".into());
    }

    tracing::error!(error = %err, "database error");
    DomainError::Validation("database operation failed".into())
}

/// Maps check constraint violations to domain errors when possible.
pub fn map_balance_constraint(err: &DbErr) -> Option<DomainError> {
    let msg = err.to_string().to_lowercase();
    if msg.contains("chk_account_balances_non_negative") {
        return Some(DomainError::NegativeBalance);
    }
    None
}
