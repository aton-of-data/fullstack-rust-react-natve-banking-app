//! SeaORM entity models for Ficus PostgreSQL tables.
//!
//! These types are an **internal** persistence concern. Callers outside this
//! crate should use application port records produced by [`crate::mapper`],
//! not `entities::*::Model` values.
//!
//! Generated SeaORM fields, `Column` enums, and `Relation` variants are
//! intentionally under-documented; see the crate root missing-docs policy.

// SeaORM DeriveEntityModel expands many public fields/associations; documenting
// each field would add noise without improving the adapter boundary.
#![allow(missing_docs)]

pub mod account_balances;
pub mod accounts;
pub mod audit_events;
pub mod idempotency_requests;
pub mod ledger_entries;
pub mod transfers;
pub mod users;

/// Convenient re-exports of SeaORM `Entity` types used inside this crate.
pub mod prelude {
    pub use super::account_balances::Entity as AccountBalance;
    pub use super::accounts::Entity as Account;
    pub use super::audit_events::Entity as AuditEvent;
    pub use super::idempotency_requests::Entity as IdempotencyRequest;
    pub use super::ledger_entries::Entity as LedgerEntry;
    pub use super::transfers::Entity as Transfer;
    pub use super::users::Entity as User;
}
