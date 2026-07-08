//! Transfer execution adapters.
//!
//! Hosts [`PostgresTransferExecutor`], the SeaORM-backed implementation of the
//! application `TransferExecutor` port. Atomic money movement, ledger writes,
//! balance projections, and idempotency storage happen here — not feed fan-out.

pub mod postgres_transfer_executor;

pub use postgres_transfer_executor::PostgresTransferExecutor;
