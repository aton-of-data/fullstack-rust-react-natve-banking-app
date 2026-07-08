//! SeaORM migration entry point for the Ficus API database.
//!
//! This crate registers schema migrations applied by the `migrate` binary and
//! by integration-test setup. It owns **DDL only** — no business logic, no
//! seed data, and no transfer/ledger orchestration.
//!
//! # Architectural role
//!
//! Schema evolution lives here so the application and domain crates stay free
//! of SQL DDL. Runtime repositories and the transfer executor assume tables and
//! constraints created by these migrations (users, accounts, balances,
//! transfers, ledger entries, idempotency requests, audit events, append-only
//! triggers, and related indexes).
//!
//! # What this crate may depend on
//!
//! - `sea-orm-migration` and supporting SeaORM migration primitives
//!
//! # What this crate must not depend on
//!
//! - Application services, HTTP adapters, or domain transfer rules
//! - Production seed credentials or funding flows (see infrastructure `seed`)
//!
//! # Neighboring crates
//!
//! - `adapters-persistence` re-exports / consumes the migrator for app and
//!   testkit startup.
//! - Infrastructure `migrate` binary connects with `DATABASE_MIGRATION_URL`
//!   (or `DATABASE_URL`) and runs [`Migrator::up`].
//!
//! Do not edit migration SQL lightly: financial invariants (append-only ledger,
//! balance constraints) are enforced partly at the database layer.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]

pub use sea_orm_migration::prelude::*;

mod m20250707_000001_create_initial_schema;

/// Registers all database migrations in apply order.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250707_000001_create_initial_schema::Migration)]
    }
}
