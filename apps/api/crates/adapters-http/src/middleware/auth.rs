//! JWT Bearer authentication middleware and extractor.
//!
//! [`require_auth`] validates `Authorization: Bearer <jwt>`, verifies the token
//! via [`AppState::tokens`], and inserts [`AuthenticatedUser`] into request
//! extensions. Handlers then extract `AuthenticatedUser` with Axum's
//! [`FromRequestParts`] — routes that need auth should sit behind this
//! middleware (as configured in [`crate::create_router`]).

use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, HeaderMap},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

/// Authenticated user identity extracted from a valid JWT.
///
/// Populated by [`require_auth`]. Extracting this type in a handler without
/// the middleware yields [`ApiError::Unauthorized`].
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// Subject user id from the verified token.
    pub user_id: Uuid,
    /// Username claim from the verified token.
    pub username: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or(ApiError::Unauthorized)
    }
}

/// JWT authentication middleware — validates Bearer token and injects [`AuthenticatedUser`].
///
/// Missing/malformed `Authorization`, empty bearer secret, or failed token
/// verification all map to [`ApiError::Unauthorized`] (HTTP 401). The raw
/// token must not be logged.
pub async fn require_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_bearer_token(request.headers())?;
    let (user_id, username) = state
        .tokens
        .verify(&token)
        .await
        .map_err(|_| ApiError::Unauthorized)?;

    request
        .extensions_mut()
        .insert(AuthenticatedUser { user_id, username });

    Ok(next.run(request).await)
}

/// Reads `Authorization: Bearer <token>` (case-insensitive `Bearer` prefix).
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, ApiError> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .ok_or(ApiError::Unauthorized)?;

    if token.is_empty() {
        return Err(ApiError::Unauthorized);
    }

    Ok(token.to_string())
}
