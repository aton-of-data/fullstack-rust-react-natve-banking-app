//! Axum middleware for auth, correlation IDs, rate limits, and metrics.
//!
//! Middleware runs outside handlers and enforces cross-cutting transport
//! policies. Re-exports below are the public adapter surface used by
//! [`crate::create_router`] and by handlers that extract typed extensions.

pub mod auth;
pub mod metrics_auth;
pub mod rate_limit;
pub mod request_id;
pub mod trace_metrics;

pub use auth::{require_auth, AuthenticatedUser};
pub use metrics_auth::require_metrics_auth;
pub use rate_limit::{login_rate_limit, transfer_rate_limit};
pub use request_id::{request_id_middleware, RequestContext, REQUEST_ID_HEADER, TRACE_ID_HEADER};
pub use trace_metrics::trace_metrics_middleware;
