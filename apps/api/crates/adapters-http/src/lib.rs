//! Ficus HTTP adapters — Axum router, middleware, and thin handlers.

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

/// Maximum JSON request body size (1 MiB).
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Builds the full Axum router with middleware, routes, and OpenAPI documentation.
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
