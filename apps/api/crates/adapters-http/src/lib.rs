//! HTTP adapter layer for the Ficus API (Axum).
//!
//! This crate translates HTTP requests into application-service calls and maps
//! domain outcomes to HTTP status codes and JSON bodies. Handlers stay thin:
//! they parse headers and bodies, enforce transport concerns (auth, idempotency
//! keys, rate limits), and delegate money-movement and auth use cases to
//! `ficus-application`.
//!
//! # Architectural role
//!
//! `ficus-adapters-http` sits at the inbound edge of the hexagonal backend. It
//! may depend on `ficus-application` (and transitively `ficus-domain` types for
//! error mapping) and on Axum / Tower HTTP middleware. It must **not** own
//! financial invariants such as double-entry ledger balancing, balance
//! deduction rules, idempotent transfer persistence, or isolation/locking
//! strategy — those live in domain and persistence.
//!
//! # What this crate may depend on
//!
//! - `ficus-application` services, ports, and DTOs
//! - `ficus-domain` errors and value types used at the boundary
//! - Axum, Tower, `tower-http`, OpenAPI (`utoipa`), metrics exporters
//!
//! # What this crate must not do
//!
//! - Open database transactions or touch SeaORM entities
//! - Implement transfer ledger writes, fingerprinting, or balance mutations
//! - Store passwords or log secrets (tokens, passwords, raw bearer headers)
//!
//! # Neighboring crates
//!
//! - **Upstream callers:** `ficus-infrastructure` (composition root) builds
//!   [`AppState`] and serves [`create_router`].
//! - **Downstream:** application services orchestrate use cases; persistence
//!   adapters perform transactional money movement.
//!
//! # Surface area
//!
//! - [`create_router`] — full `/v1` API, health, metrics, optional Swagger UI
//! - [`AppState`] — shared service handles and runtime knobs
//! - [`ApiError`] / [`ErrorBody`] — safe client-facing error mapping
//! - Middleware: JWT auth, request/trace IDs, rate limits, metrics auth, HTTP metrics
//! - Handlers: auth, users, accounts, transfers, feed, health

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]

mod error;
mod handlers;
mod metrics;
mod middleware;
mod openapi;
mod state;

pub use error::{ApiError, ErrorBody};
pub use metrics::{init_metrics, metrics_handler, prometheus_handle};
pub use middleware::{
    login_rate_limit, require_auth, require_metrics_auth, transfer_rate_limit, AuthenticatedUser,
    RequestContext, REQUEST_ID_HEADER, TRACE_ID_HEADER,
};
pub use openapi::ApiDoc;
pub use state::{AppState, ReadinessCheck};

use std::time::Duration;

use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::middleware::rate_limit::RateLimiter;
use crate::middleware::{request_id_middleware, trace_metrics_middleware};
use crate::state::AppState as State;

/// Maximum accepted HTTP request body size (1 MiB).
///
/// Applied globally via [`RequestBodyLimitLayer`]. Oversized JSON bodies are
/// rejected before handlers run.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Builds the full Axum router with middleware, `/v1` routes, health, and metrics.
///
/// # Layers and routes (outer → inner)
///
/// Global layers (applied to every request):
/// - Request/trace ID injection (`request_id_middleware`)
/// - HTTP request metrics (`trace_metrics_middleware`)
/// - Tower `TraceLayer`
/// - CORS from private `cors_layer`
/// - Body size limit ([`MAX_BODY_BYTES`])
/// - Security headers (`X-Content-Type-Options`, `X-Frame-Options`, etc.)
///
/// Route groups:
/// - `GET /health/live`, `GET /health/ready` — unauthenticated probes
/// - `GET /metrics` — Prometheus scrape; gated by [`require_metrics_auth`]
/// - `/v1/*` — versioned API; login is public (rate-limited); other routes
///   require JWT via [`require_auth`]
/// - `/api-docs` + OpenAPI JSON — only when `environment` is `development` or
///   `test`
///
/// Transfer and login routes attach additional per-route rate-limit layers.
///
/// # Responsibilities
///
/// Initializes Prometheus metrics (best-effort), wires rate limiters from
/// [`AppState`] knobs, and returns a stateful [`Router`]. Callers from the
/// infrastructure crate bind this router to a listener.
pub fn create_router(state: State) -> Router {
    if let Err(err) = init_metrics() {
        tracing::error!(error = %err, "failed to initialize Prometheus metrics");
    }

    let login_limiter = RateLimiter::new(state.login_rate_limit_per_min, Duration::from_secs(60));
    let transfer_limiter =
        RateLimiter::new(state.transfer_rate_limit_per_min, Duration::from_secs(60));
    let trust_proxy_headers = state.trust_proxy_headers;

    let authenticated = Router::new()
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/me", get(handlers::auth::me))
        .route("/users", get(handlers::users::search_users))
        .route("/accounts/me/balance", get(handlers::accounts::get_balance))
        .route("/accounts/me/ledger", get(handlers::accounts::get_ledger))
        .route(
            "/transfers",
            post(handlers::transfers::create_transfer).route_layer(from_fn_with_state(
                transfer_limiter.clone(),
                transfer_rate_limit,
            )),
        )
        .route("/feed", get(handlers::feed::list_feed))
        .route("/feed/stream", get(handlers::feed::stream_feed))
        .route_layer(from_fn_with_state(state.clone(), require_auth));

    let v1 = Router::new()
        .route(
            "/auth/login",
            post(handlers::auth::login).route_layer(from_fn_with_state(
                (login_limiter, trust_proxy_headers),
                login_rate_limit,
            )),
        )
        .merge(authenticated);

    let mut router = Router::new()
        .route("/health/live", get(handlers::health::live))
        .route("/health/ready", get(handlers::health::ready))
        .route(
            "/metrics",
            get(metrics_handler)
                .route_layer(from_fn_with_state(state.clone(), require_metrics_auth)),
        )
        .nest("/v1", v1);

    if matches!(state.environment.as_str(), "development" | "test") {
        router = router
            .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    let cors_origins = state.cors_origins.clone();

    router
        .with_state(state)
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::REFERRER_POLICY,
            axum::http::HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("permissions-policy"),
            axum::http::HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        ))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(cors_layer(&cors_origins))
        .layer(TraceLayer::new_for_http())
        .layer(from_fn(trace_metrics_middleware))
        .layer(from_fn(request_id_middleware))
}

/// Builds a CORS layer for configured browser origins.
///
/// Allows `GET`, `POST`, and `OPTIONS` with any request headers. When
/// `origins` is empty or contains no parseable values, falls back to
/// `http://localhost` so local clients can call the API. Production must pass
/// explicit allowed origin strings via [`AppState::cors_origins`].
fn cors_layer(origins: &[String]) -> CorsLayer {
    let layer = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(Any);

    let parsed: Vec<axum::http::HeaderValue> = origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    if parsed.is_empty() {
        layer.allow_origin(AllowOrigin::list(vec![
            axum::http::HeaderValue::from_static("http://localhost"),
        ]))
    } else {
        layer.allow_origin(AllowOrigin::list(parsed))
    }
}
