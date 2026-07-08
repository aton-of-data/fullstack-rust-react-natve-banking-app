//! HTTP request handlers for the Ficus API.
//!
//! Each submodule exposes thin Axum handlers registered by [`crate::create_router`].
//! Handlers convert transport concerns (auth extractors, headers, query params,
//! JSON bodies) into application-service calls and map failures through
//! [`crate::ApiError`]. They must not open database transactions or enforce
//! money-movement invariants beyond what the application layer already provides.
//!
//! # Modules
//!
//! - [`auth`] — login, logout, and current-user profile
//! - [`users`] — username prefix search
//! - [`accounts`] — balance and ledger history
//! - [`transfers`] — authenticated money transfer creation
//! - [`feed`] — paginated and SSE live transaction feed
//! - [`health`] — liveness and readiness probes

pub mod accounts;
pub mod auth;
pub mod feed;
pub mod health;
pub mod transfers;
pub mod users;
