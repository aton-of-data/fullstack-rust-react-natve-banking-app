//! Financial account aggregates.
//!
//! This module belongs to the domain layer. An [`Account`] is owned by a user
//! or is the system funding account used to seed initial balances. Balance
//! projections and ledger entries reference account ids; this type does not
//! store balances.

use uuid::Uuid;

/// A financial account owned by a user (or the system funding account).
///
/// User accounts fund P2P transfers. The system funding account is used by seed
/// and funding flows to inject initial liquidity without inventing money inside
/// user-to-user transfers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// Account primary key.
    pub id: Uuid,
    /// Owning user id, or nil UUID for the system funding account.
    pub user_id: Uuid,
    /// When true, this is the system funding account (`user_id` is nil).
    pub is_system: bool,
}

impl Account {
    /// Creates a regular user account.
    pub fn user_account(id: Uuid, user_id: Uuid) -> Self {
        Self {
            id,
            user_id,
            is_system: false,
        }
    }

    /// Creates the system funding account.
    ///
    /// Used by seed/funding paths. Transfers between end users must not treat
    /// this as a normal recipient unless explicitly implementing funding.
    pub fn system_funding(id: Uuid) -> Self {
        Self {
            id,
            user_id: Uuid::nil(),
            is_system: true,
        }
    }
}
