use axum::{extract::State, http::StatusCode, Json};
use ficus_application::dto::{LoginRequest, LoginResponse, MeResponse};

use crate::error::ApiError;
use crate::metrics::record_login_attempt;
use crate::middleware::{AuthenticatedUser, RequestContext};
use crate::state::AppState;

/// Authenticates a user and returns a JWT access token.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = crate::error::ErrorBody),
        (status = 429, description = "Rate limited", body = crate::error::ErrorBody),
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    ctx: axum::Extension<RequestContext>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let result = state
        .auth
        .login(
            &body.username,
            &body.password,
            &ctx.request_id,
            &ctx.trace_id,
        )
        .await;

    match &result {
        Ok(_) => record_login_attempt(true),
        Err(ficus_domain::errors::DomainError::InvalidCredentials) => record_login_attempt(false),
        _ => {}
    }

    let (access_token, user_id, username) = result?;
    Ok(Json(LoginResponse {
        access_token,
        user_id,
        username,
    }))
}

/// Logs out the current session (client should discard the token).
#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    responses(
        (status = 204, description = "Logged out"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn logout(_user: AuthenticatedUser) -> StatusCode {
    StatusCode::NO_CONTENT
}

/// Returns the authenticated user's profile.
#[utoipa::path(
    get,
    path = "/v1/auth/me",
    responses(
        (status = 200, description = "Current user", body = MeResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<MeResponse>, ApiError> {
    let (user_id, username) = state.auth.me(user.user_id).await?;
    Ok(Json(MeResponse { user_id, username }))
}
