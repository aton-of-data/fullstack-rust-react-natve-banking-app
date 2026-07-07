use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::state::AppState;

/// Liveness probe response body.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// Returns 200 when the process is running.
#[utoipa::path(
    get,
    path = "/health/live",
    responses((status = 200, description = "Process is alive", body = HealthResponse)),
    tag = "health"
)]
pub async fn live() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// Returns 200 when dependencies are ready, 503 otherwise.
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
