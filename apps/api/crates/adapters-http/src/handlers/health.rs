//! Process health probe HTTP handlers.
//!
//! Unauthenticated endpoints used by orchestrators and load balancers.
//! Liveness only checks that the process responds; readiness optionally
//! consults [`crate::ReadinessCheck`] (typically database reachability).

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::state::AppState;

/// Liveness / readiness JSON body.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    /// `"ok"` when the probe succeeds; `"not_ready"` for failed readiness.
    pub status: &'static str,
}

/// `GET /health/live` — liveness probe.
///
/// # Auth
///
/// None. Always returns 200 with `{ "status": "ok" }` while the process is up.
#[utoipa::path(
    get,
    path = "/health/live",
    responses((status = 200, description = "Process is alive", body = HealthResponse)),
    tag = "health"
)]
pub async fn live() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// `GET /health/ready` — readiness probe.
///
/// # Auth
///
/// None. Returns 200 when dependencies are ready (or when no
/// [`crate::ReadinessCheck`] is configured). Returns 503 with
/// `"not_ready"` when the check reports unavailable.
#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "Ready to serve traffic", body = HealthResponse),
        (status = 503, description = "Not ready", body = HealthResponse),
    ),
    tag = "health"
)]
pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let ready = match &state.readiness {
        Some(check) => check.is_ready().await,
        None => true,
    };

    if ready {
        (StatusCode::OK, Json(HealthResponse { status: "ok" }))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "not_ready",
            }),
        )
    }
}
