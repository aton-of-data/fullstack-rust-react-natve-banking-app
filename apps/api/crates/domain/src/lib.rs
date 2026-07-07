//! Ficus domain layer — aggregates, value objects, and pure business rules.

pub mod account;
pub mod audit;
pub mod currency;
pub mod errors;
pub mod idempotency;
pub mod ledger;
pub mod money;
pub mod transfer;
pub mod user;

pub use account::Account;
pub use currency::Currency;
pub use errors::DomainError;
pub use ledger::{LedgerDirection, LedgerEntryDraft};
pub use money::Money;
pub use transfer::{Transfer, TransferStatus};
pub use user::User;
