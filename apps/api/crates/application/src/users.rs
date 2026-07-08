//! User search, balance, and ledger read use cases.
//!
//! [`UserService`] is a thin orchestration layer over [`UserRepository`] and
//! [`AccountRepository`]. It validates search query bounds and resolves the
//! caller's account before balance/ledger reads. It does not mutate money,
//! create users, or perform authentication.
//!
//! # Non-responsibilities
//!
//! - Password handling (see [`crate::auth::AuthService`])
//! - Transfer execution (see [`crate::transfer::TransferService`])
//! - HTTP authorization — callers must already know `user_id` / `exclude_user_id`

use std::sync::Arc;

use ficus_domain::errors::DomainError;
use uuid::Uuid;

use crate::ports::{AccountRepository, Page, UserRepository};

/// Read-side use cases for discovering users and inspecting own funds history.
pub struct UserService {
    users: Arc<dyn UserRepository>,
    accounts: Arc<dyn AccountRepository>,
}

impl UserService {
    /// Creates a user service over user and account ports.
    pub fn new(users: Arc<dyn UserRepository>, accounts: Arc<dyn AccountRepository>) -> Self {
        Self { users, accounts }
    }

    /// Searches users by username query, excluding `exclude_user_id`.
    ///
    /// # Validation
    ///
    /// Rejects empty/whitespace-only queries and queries longer than 32
    /// characters with [`DomainError::Validation`]. Remaining filtering and
    /// pagination are delegated to [`UserRepository::search_by_username`].
    ///
    /// Returned [`UserRecord`](crate::ports::UserRecord) values may include
    /// `password_hash` from persistence; HTTP adapters must map to
    /// [`crate::dto::UserSearchItem`] (public fields only) before responding.
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

    /// Returns the balance projection for `user_id`'s primary account.
    ///
    /// Resolves the account via [`AccountRepository::find_by_user_id`], then
    /// loads [`AccountRepository::get_balance`]. Missing accounts become
    /// [`DomainError::AccountNotFound`].
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

    /// Returns cursor-paginated ledger history for `user_id`'s primary account.
    ///
    /// Same account-resolution rules as [`Self::get_balance`]. Ordering and
    /// cursor encoding belong to the account repository implementor.
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
