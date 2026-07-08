//! Transfer aggregate and lifecycle status.
//!
//! This module belongs to the domain layer. It models the money-movement
//! transfer record shape used after business validation. Persistence assigns
//! identifiers and timestamps; this type does not open transactions or write
//! ledger rows.

use uuid::Uuid;

/// Transfer lifecycle status persisted with the transfer row.
///
/// Successful money movement commits as [`TransferStatus::Completed`]. The
/// domain also defines [`TransferStatus::Declined`] for explicit decline
/// recording; the current executor typically aborts on insufficient funds
/// without inserting a declined transfer row (audit may still record the
/// decline outside the aborted transaction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransferStatus {
    /// Transfer committed with balanced ledger entries and updated balances.
    Completed,
    /// Transfer explicitly declined (e.g. policy); not a successful movement.
    Declined,
}

/// A money transfer between two accounts.
///
/// Amounts are integer minor units. `currency_code` mirrors the booked
/// currency (e.g. `"USD"`). This is a domain snapshot; the application layer
/// transfer record type adds usernames and timestamps for API responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    /// Stable transfer identifier.
    pub id: Uuid,
    /// Debited account.
    pub sender_account_id: Uuid,
    /// Credited account.
    pub recipient_account_id: Uuid,
    /// Positive amount in minor units.
    pub amount_minor: i64,
    /// ISO currency code string.
    pub currency_code: String,
    /// Optional free-text description supplied by the client.
    pub description: Option<String>,
    /// Lifecycle status.
    pub status: TransferStatus,
}
