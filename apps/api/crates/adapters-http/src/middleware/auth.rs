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
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
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

/// JWT authentication middleware — validates Bearer token and injects `AuthenticatedUser`.
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
