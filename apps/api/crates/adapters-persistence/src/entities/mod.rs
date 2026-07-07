//! SeaORM entity models for Ficus persistence.

pub mod account_balances;
pub mod accounts;
pub mod audit_events;
pub mod idempotency_requests;
pub mod ledger_entries;
pub mod transfers;
pub mod users;

pub mod prelude {
    pub use super::account_balances::Entity as AccountBalance;
    pub use super::accounts::Entity as Account;
    pub use super::audit_events::Entity as AuditEvent;
    pub use super::idempotency_requests::Entity as IdempotencyRequest;
    pub use super::ledger_entries::Entity as LedgerEntry;
    pub use super::transfers::Entity as Transfer;
    pub use super::users::Entity as User;
}
