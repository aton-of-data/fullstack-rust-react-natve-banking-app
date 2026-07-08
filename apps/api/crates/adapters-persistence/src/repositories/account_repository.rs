//! Account and ledger query repository.
//!
//! **Tables:** `accounts`, `account_balances`, `ledger_entries`.
//!
//! Resolves a user's account id, reads balance projections, and pages ledger
//! history (newest-first cursor). Does not mutate balances — the transfer
//! executor owns money writes.

use async_trait::async_trait;
use ficus_application::ports::{AccountRepository, BalanceRecord, LedgerEntryRecord, Page};
use ficus_domain::errors::DomainError;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::entities::account_balances::Entity as AccountBalance;
use crate::entities::accounts::{self, Entity as Account};
use crate::entities::ledger_entries::{self, Entity as LedgerEntry};
use crate::error::map_db_err;
use crate::mapper::{balance_to_record, decode_cursor, encode_cursor, ledger_entry_to_record};

/// SeaORM-backed account repository (`accounts`, `account_balances`, `ledger_entries`).
pub struct PostgresAccountRepository {
    db: DatabaseConnection,
}

impl PostgresAccountRepository {
    /// Creates a repository backed by the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Uuid>, DomainError> {
        let row = Account::find()
            .filter(accounts::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(row.map(|account| account.id))
    }

    async fn get_balance(&self, account_id: Uuid) -> Result<BalanceRecord, DomainError> {
        let row = AccountBalance::find_by_id(account_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::AccountNotFound)?;
        Ok(balance_to_record(row))
    }

    async fn get_ledger(
        &self,
        account_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<LedgerEntryRecord>, DomainError> {
        let mut condition = Condition::all().add(ledger_entries::Column::AccountId.eq(account_id));

        if let Some(cursor) = cursor {
            let (created_at, id) = decode_cursor(cursor)?;
            let ts: sea_orm::prelude::DateTimeWithTimeZone = created_at.into();
            condition = condition.add(
                Condition::any()
                    .add(ledger_entries::Column::CreatedAt.lt(ts))
                    .add(
                        Condition::all()
                            .add(ledger_entries::Column::CreatedAt.eq(ts))
                            .add(ledger_entries::Column::Id.lt(id)),
                    ),
            );
        }

        let rows = LedgerEntry::find()
            .filter(condition)
            .order_by_desc(ledger_entries::Column::CreatedAt)
            .order_by_desc(ledger_entries::Column::Id)
            .limit(limit + 1)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;

        let has_more = rows.len() > limit as usize;
        let items: Vec<LedgerEntryRecord> = rows
            .into_iter()
            .take(limit as usize)
            .map(ledger_entry_to_record)
            .collect();

        let next_cursor = if has_more {
            items
                .last()
                .map(|entry| encode_cursor(entry.created_at, entry.id))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }
}
