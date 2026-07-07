use std::sync::Arc;

use ficus_domain::errors::DomainError;
use uuid::Uuid;

use crate::ports::{AccountRepository, Page, UserRepository};

/// User search and profile use cases.
pub struct UserService {
    users: Arc<dyn UserRepository>,
    accounts: Arc<dyn AccountRepository>,
}

impl UserService {
    pub fn new(users: Arc<dyn UserRepository>, accounts: Arc<dyn AccountRepository>) -> Self {
        Self { users, accounts }
    }

    /// Searches users by username prefix.
    pub async fn search(
        &self,
        query: &str,
        exclude_user_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<crate::ports::UserRecord>, DomainError> {
        if query.trim().is_empty() || query.len() > 32 {
            return Err(DomainError::Validation(
                "query must be 1-32 characters".into(),
            ));
        }
        self.users
            .search_by_username(query, exclude_user_id, cursor, limit)
            .await
    }

    /// Returns current user balance.
    pub async fn get_balance(
        &self,
        user_id: Uuid,
    ) -> Result<crate::ports::BalanceRecord, DomainError> {
        let account_id = self
            .accounts
            .find_by_user_id(user_id)
            .await?
            .ok_or(DomainError::AccountNotFound)?;
        self.accounts.get_balance(account_id).await
    }

    /// Returns ledger history for current user.
    pub async fn get_ledger(
        &self,
        user_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<crate::ports::LedgerEntryRecord>, DomainError> {
        let account_id = self
            .accounts
            .find_by_user_id(user_id)
            .await?
            .ok_or(DomainError::AccountNotFound)?;
        self.accounts.get_ledger(account_id, cursor, limit).await
    }
}
