//! PostgreSQL persistence adapters for the Ficus API.

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
