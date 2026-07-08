//! Shared balance and ledger assertion helpers for integration tests.
//!
//! These helpers express the financial **invariants** exercised by the
//! concurrency, conservation, and reconciliation suites:
//!
//! - [`negative_balances`] — no projection may be strictly negative.
//! - [`total_balance_minor`] — sum of all balance projections (conservation of
//!   system money including the system account).
//! - [`orphan_ledger_entries`] — every ledger row must reference a
//!   **completed** transfer (no partial / rolled-back leftovers).
//! - [`reconcile_all_accounts`] / [`ledger_derived_balance`] — non-system
//!   projections must equal append-only ledger reconstruction.

use ficus_adapters_persistence::entities::{account_balances, accounts, ledger_entries, transfers};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

/// Signed ledger amount: debits negative, credits positive.
pub fn signed_ledger_amount(direction: &str, amount_minor: i64) -> i64 {
    match direction {
        "debit" => -amount_minor,
        "credit" => amount_minor,
        other => panic!("unknown ledger direction: {other}"),
    }
}

/// Sums all account balance projections in minor units.
///
/// Includes the system account. Successful user↔user transfers must leave this
/// total unchanged (money conserved). Failure to conserve means a bug in
/// debit/credit application or a race that overwrote balances.
pub async fn total_balance_minor(db: &DatabaseConnection) -> Result<i64, sea_orm::DbErr> {
    let rows = account_balances::Entity::find().all(db).await?;
    Ok(rows.iter().map(|row| row.balance_minor).sum())
}

/// Reconstructs an account balance from append-only ledger entries.
pub async fn ledger_derived_balance(
    db: &DatabaseConnection,
    account_id: Uuid,
) -> Result<i64, sea_orm::DbErr> {
    let entries = ledger_entries::Entity::find()
        .filter(ledger_entries::Column::AccountId.eq(account_id))
        .all(db)
        .await?;
    Ok(entries
        .iter()
        .map(|entry| signed_ledger_amount(&entry.direction, entry.amount_minor))
        .sum())
}

/// Returns balances that are strictly negative.
///
/// Invariant: production transfers must never leave a negative projection.
/// A non-empty result means locking / insufficient-funds checks failed under
/// concurrency or a bypass helper left an invalid state.
pub async fn negative_balances(
    db: &DatabaseConnection,
) -> Result<Vec<(Uuid, i64)>, sea_orm::DbErr> {
    let rows = account_balances::Entity::find().all(db).await?;
    Ok(rows
        .iter()
        .filter(|row| row.balance_minor < 0)
        .map(|row| (row.account_id, row.balance_minor))
        .collect())
}

/// Counts completed transfers in persistence.
pub async fn count_completed_transfers(db: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    Ok(transfers::Entity::find()
        .filter(transfers::Column::Status.eq("completed"))
        .all(db)
        .await?
        .len() as u64)
}

/// Counts all transfer rows regardless of status.
pub async fn count_all_transfers(db: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    Ok(transfers::Entity::find().all(db).await?.len() as u64)
}

/// Verifies every ledger entry references a completed transfer.
///
/// Invariant: failed/rolled-back transfers must not leave ledger rows; entries
/// pointing at missing or non-completed transfers are orphans (partial state).
pub async fn orphan_ledger_entries(db: &DatabaseConnection) -> Result<Vec<Uuid>, sea_orm::DbErr> {
    let entries = ledger_entries::Entity::find().all(db).await?;
    let mut orphans = Vec::new();
    for entry in entries {
        let transfer = transfers::Entity::find_by_id(entry.transfer_id)
            .one(db)
            .await?;
        let orphan = match transfer {
            None => true,
            Some(t) => t.status != "completed",
        };
        if orphan {
            orphans.push(entry.id);
        }
    }
    Ok(orphans)
}

/// Reconciliation mismatch for a single account.
#[derive(Debug, Clone)]
pub struct ReconciliationMismatch {
    /// Account identifier.
    pub account_id: Uuid,
    /// Materialized projection balance.
    pub projected_balance_minor: i64,
    /// Balance derived from ledger entries.
    pub ledger_balance_minor: i64,
}

/// Reconciles all account balance projections against ledger history.
///
/// Skips the system account (seed funding may leave intentional projection
/// differences vs a full double-entry history depending on seed path). For
/// user accounts, any mismatch means the projection and append-only ledger
/// diverged — a critical money-integrity failure.
pub async fn reconcile_all_accounts(
    db: &DatabaseConnection,
) -> Result<Vec<ReconciliationMismatch>, sea_orm::DbErr> {
    let rows = account_balances::Entity::find().all(db).await?;
    let mut mismatches = Vec::new();
    for row in rows {
        let account = accounts::Entity::find_by_id(row.account_id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom("account missing for balance row".into()))?;
        if account.is_system {
            continue;
        }
        let ledger_balance = ledger_derived_balance(db, row.account_id).await?;
        if ledger_balance != row.balance_minor {
            mismatches.push(ReconciliationMismatch {
                account_id: row.account_id,
                projected_balance_minor: row.balance_minor,
                ledger_balance_minor: ledger_balance,
            });
        }
    }
    Ok(mismatches)
}

/// Counts idempotency records in persistence.
pub async fn count_idempotency_records(db: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    use ficus_adapters_persistence::entities::idempotency_requests;
    Ok(idempotency_requests::Entity::find().all(db).await?.len() as u64)
}

/// Counts ledger entries in persistence.
pub async fn count_ledger_entries(db: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    Ok(ledger_entries::Entity::find().all(db).await?.len() as u64)
}

/// Counts audit events in persistence.
pub async fn count_audit_events(db: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    use ficus_adapters_persistence::entities::audit_events;
    Ok(audit_events::Entity::find().all(db).await?.len() as u64)
}
