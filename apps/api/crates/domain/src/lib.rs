//! Domain model for the Ficus money movement system.
//!
//! This crate contains pure business rules for money, currency, ledger entries,
//! transfer status, idempotency fingerprints, users/accounts, and audit event
//! drafts. All amounts are integer minor units; floating-point arithmetic is
//! prohibited.
//!
//! # Architectural role
//!
//! `ficus-domain` is the innermost crate. It must not depend on HTTP,
//! PostgreSQL, SeaORM, JWT, environment configuration, tracing, or any runtime
//! infrastructure.
//!
//! # What this crate may depend on
//!
//! - Pure Rust standard library
//! - Small value-oriented crates (`uuid`, `chrono`, `thiserror`, `sha2`, `hex`,
//!   `serde` for status serialization)
//!
//! # What this crate must not depend on
//!
//! - `axum`, SeaORM, SQL, JWT, Argon2, metrics exporters, or config loaders
//! - Application ports or adapter crates
//!
//! # Financial invariants
//!
//! - Money is represented as non-negative integer minor units (except signed
//!   ledger reconstruction helpers that sum debits/credits).
//! - Transfer amounts must be strictly positive.
//! - Completed transfer ledger entry pairs must balance to zero.
//! - Domain validation never performs I/O or opens database transactions.
//!
//! # Neighboring crates
//!
//! The application crate orchestrates use cases using these types.
//! Persistence and HTTP adapters translate external representations into this
//! domain model at system boundaries.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]

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
