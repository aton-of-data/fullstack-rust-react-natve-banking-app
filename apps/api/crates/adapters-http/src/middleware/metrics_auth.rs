//! Authorization for the Prometheus `/metrics` scrape endpoint.
//!
//! When a metrics bearer token is configured, or when the process is not in
//! `development`/`test`, scrapes must present `Authorization: Bearer <token>`.
//! Without a configured token outside those environments, the endpoint returns
//! 404 (disabled) rather than exposing metrics anonymously.

use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::state::AppState;

/// Protects `/metrics` when a bearer token is configured or in non-dev environments.
///
/// # Policy
///
/// - `development` / `test` **and** no `metrics_auth_token` → open scrape
/// - Token configured → require matching Bearer token (401 on mismatch)
/// - Non-dev without token → 404 disabled
pub async fn require_metrics_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    if !metrics_endpoint_requires_auth(&state) {
        return Ok(next.run(request).await);
    }

    let Some(expected) = state.metrics_auth_token.as_deref() else {
        return Err((
            StatusCode::NOT_FOUND,
            "metrics endpoint disabled in this environment",
        )
            .into_response());
    };

    let authorized = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| token == expected);

    if authorized {
        Ok(next.run(request).await)
    } else {
        Err((StatusCode::UNAUTHORIZED, "metrics access denied").into_response())
    }
}

/// Whether `/metrics` must authenticate for this [`AppState`].
fn metrics_endpoint_requires_auth(state: &AppState) -> bool {
    state.metrics_auth_token.is_some()
        || !matches!(state.environment.as_str(), "development" | "test")
}
