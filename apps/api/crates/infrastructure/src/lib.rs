//! Ficus infrastructure — composition root, config, auth adapters, telemetry.
//!
//! This crate is the **runtime glue** for the API process: load configuration,
//! wire ports to Postgres/HTTP adapters, initialize logging, and expose
//! binaries (`ficus-api`, `migrate`, `seed`).
//!
//! # Architectural role
//!
//! Outermost composition root. It may depend on application ports and adapter
//! crates, but must not redefine domain financial invariants. Money movement
//! rules stay in `domain` / `application` / `adapters-persistence`.
//!
//! # What this crate may depend on
//!
//! - `ficus-application`, `ficus-adapters-http`, `ficus-adapters-persistence`
//! - Config (`dotenvy`), JWT/Argon2 adapters, SeaORM connection, tracing
//!
//! # What this crate must not depend on
//!
//! - Embedding business transfer logic beyond wiring ports
//! - Shipping secrets into source control (load from environment only)
//!
//! # Modules
//!
//! - [`auth`] — Argon2 password hashing and JWT token service adapters
//! - [`config`] — [`AppConfig`] from environment variables
//! - [`readiness`] — database ping for `/ready` probes
//! - [`telemetry`] — structured logging / OTEL hooks
//! - [`wiring`] — [`build_app`] composition of repositories and services
//!
//! # Neighboring crates
//!
//! - Binaries in this crate start the HTTP server or run migrate/seed.
//! - `testkit` reuses auth and wiring helpers for integration tests.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]

pub mod auth;
pub mod config;
pub mod readiness;
pub mod telemetry;
pub mod wiring;

pub use config::AppConfig;
pub use telemetry::{init_telemetry, shutdown_telemetry};
pub use wiring::build_app;
