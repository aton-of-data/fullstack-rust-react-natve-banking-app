//! Authentication HTTP handlers.
//!
//! Public login issues a JWT access token. Logout and `/me` require a valid
//! Bearer token via [`crate::require_auth`]. Credential verification and token
//! signing are owned by the application/auth ports — this module only bridges
//! HTTP.
//!
//! # Security notes
//!
//! - Failed logins return a **generic** invalid-credentials message so
//!   responses do not distinguish unknown usernames from bad passwords.
//! - Passwords and tokens must never be written to logs; metrics record only
//!   success/failure counts.
//! - Logout is a **client-side** token discard: the server returns 204 and does
//!   not maintain a server-side session blocklist in this adapter.

use axum::{extract::State, http::StatusCode, Json};
use ficus_application::dto::{LoginRequest, LoginResponse, MeResponse};

use crate::error::ApiError;
use crate::metrics::record_login_attempt;
use crate::middleware::{AuthenticatedUser, RequestContext};
use crate::state::AppState;

/// `POST /v1/auth/login` — authenticate with username and password.
///
/// # Auth requirements
///
/// **Unauthenticated.** Protected only by the login rate-limiter (per client
/// IP). Does not require a Bearer token.
///
/// # Request / response
///
/// Accepts JSON [`LoginRequest`] (`username`, `password`). On success returns
/// [`LoginResponse`] containing a JWT `access_token` plus `user_id` and
/// `username`. Clients must send that JWT as `Authorization: Bearer <token>`
/// on subsequent protected routes.
///
/// # Errors
///
/// Invalid username or password maps to [`ApiError`] /
/// [`DomainError::InvalidCredentials`](ficus_domain::errors::DomainError::InvalidCredentials)
/// → HTTP 401 with a generic `"Invalid username or password"` message (no
/// user enumeration). Rate limit → 429.
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

/// `POST /v1/auth/logout` — end the client session.
///
/// # Auth requirements
///
/// Requires a valid Bearer JWT ([`AuthenticatedUser`]). Invalid/missing token
/// → 401 before this handler runs.
///
/// # Behavior
///
/// Returns **204 No Content**. There is no server-side session store to
/// invalidate in this API; clients must discard the access token locally.
/// Calling logout is still useful for uniform client flows and to confirm the
/// token was valid at logout time.
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

/// `GET /v1/auth/me` — return the authenticated user's profile.
///
/// # Auth requirements
///
/// Requires a valid Bearer JWT. Resolves the current user via
/// [`AuthenticatedUser::user_id`] and the auth application service.
///
/// # Response
///
/// JSON [`MeResponse`] with `user_id` and `username`. Returns 401 when
/// unauthenticated; 404-style domain errors if the user record is missing.
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
