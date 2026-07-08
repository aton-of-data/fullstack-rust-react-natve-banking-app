//! PostgreSQL repository implementations of application ports.
//!
//! Each submodule maps to one or more SeaORM entities / tables and returns
//! application records via [`crate::mapper`]. Repositories are read/append
//! helpers; the transfer money path lives in [`crate::executor`].

mod account_repository;
mod audit_repository;
mod idempotency_repository;
mod transfer_repository;
mod user_repository;

pub use account_repository::PostgresAccountRepository;
pub use audit_repository::PostgresAuditRepository;
pub use idempotency_repository::PostgresIdempotencyRepository;
pub use transfer_repository::PostgresTransferRepository;
pub use user_repository::PostgresUserRepository;
