//! Application layer — use cases, DTOs, and repository ports.
//!
//! This crate orchestrates authentication, transfers, user search, and feed
//! queries. It depends only on [`ficus_domain`] for business rules and defines
//! async port traits implemented by persistence and infrastructure adapters.
//!
//! # Architectural role
//!
//! Hexagonal **application** ring: coordinates use cases without SQL, Axum
//! types, or SeaORM entities. Side effects occur only through port traits.
//!
//! # What this crate may depend on
//!
//! - `ficus-domain`
//! - Async trait utilities, serde, metrics/tracing façades used at orchestration
//!   boundaries (not database drivers)
//!
//! # What this crate must not depend on
//!
//! - `axum`, SeaORM, SQL, JWT libraries, Argon2, or HTTP status enums
//! - Persistence entity modules
//!
//! # Important invariants
//!
//! - Reducers are not used here (backend); pure domain validation still applies
//!   before calling executors.
//! - [`TransferService`] validates idempotency key format and fingerprints
//!   requests, then delegates transactional money movement to
//!   [`ports::TransferExecutor`].
//! - Feed publication happens **after** a successful completed transfer return
//!   from the executor; feed failures are logged and do not roll back money.
//!
//! # Neighboring crates
//!
//! - `adapters-http` calls these services from handlers.
//! - `adapters-persistence` implements port traits and the transfer executor.
//! - `infrastructure` wires concrete adapters into services at startup.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]

pub mod auth;
pub mod dto;
pub mod feed;
pub mod ports;
pub mod transfer;
pub mod users;

pub use auth::AuthService;
pub use feed::FeedService;
pub use transfer::TransferService;
pub use users::UserService;
