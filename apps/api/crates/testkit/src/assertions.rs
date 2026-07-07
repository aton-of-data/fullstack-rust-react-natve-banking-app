//! Shared balance and ledger assertion helpers for integration tests.

use ficus_adapters_persistence::entities::{account_balances, ledger_entries, transfers};
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

/// Verifies every ledger entry references a completed transfer.
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
