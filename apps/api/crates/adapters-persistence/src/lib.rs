//! PostgreSQL / SeaORM persistence adapters for the Ficus API.
//!
//! This crate implements application ports (`TransferExecutor`, repositories,
//! `FeedBroadcaster`) against PostgreSQL. It owns SeaORM entities, mappers that
//! convert ORM rows into application records, and the atomic transfer executor.
//!
//! # Architectural role
//!
//! `ficus-adapters-persistence` is an **adapter** (driven / secondary) layer in
//! the hexagonal layout. Higher layers must depend on port traits from
//! `ficus-application`, not on SeaORM types from this crate.
//!
//! # What this crate may depend on
//!
//! - `ficus-domain` and `ficus-application` (ports + record DTOs)
//! - `ficus-migrations` (re-exported migrator for composition roots)
//! - SeaORM, sqlx, Tokio, chrono, uuid, serde/serde_json, metrics, tracing
//!
//! # What this crate must not depend on
//!
//! - `axum` / HTTP handler types (`adapters-http` concerns)
//! - Domain business rules beyond calling domain helpers (e.g. `Money`,
//!   `build_transfer_entries`) — keep orchestration logic in application
//! - Leaking `entities::*` models through public port return types (map via
//!   [`mapper`] first)
//!
//! # Invariants enforced here
//!
//! - Transfers run in a single PostgreSQL transaction with **ReadCommitted**
//!   isolation, `FOR UPDATE` balance locks, and transactional advisory locks
//!   for idempotency (see [`executor::PostgresTransferExecutor`]).
//! - Balance rows are locked in deterministic UUID order to avoid deadlocks.
//! - Successful transfers write one transfer row, exactly two ledger entries,
//!   updated balance projections, and an idempotency response in the same txn.
//! - Insufficient-funds failures roll back the txn (no transfer/ledger rows);
//!   declined audit is written **outside** the aborted transaction.
//! - Feed publication is **not** performed inside the transfer executor; the
//!   application layer publishes after a successful execute.
//!
//! # Neighboring crates
//!
//! - `ficus-application` — port traits and use cases that call these adapters
//! - `ficus-adapters-http` — HTTP surface; must not import SeaORM entities
//! - `ficus-infrastructure` — wires connections and constructs repositories
//!
//! # Missing-docs policy
//!
//! Crate-level `missing_docs` is enabled. SeaORM entity modules generate many
//! public fields/`Column`/`Relation` items; those modules allow the lint with
//! an explicit reason. Public adapter APIs and mappers must stay documented.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]

pub mod entities;
pub mod error;
pub mod executor;
pub mod feed;
pub mod mapper;
pub mod repositories;

pub use executor::PostgresTransferExecutor;
pub use feed::InMemoryFeedBroadcaster;
pub use ficus_migrations::Migrator;
pub use repositories::{
    PostgresAccountRepository, PostgresAuditRepository, PostgresIdempotencyRepository,
    PostgresTransferRepository, PostgresUserRepository,
};
