//! User search HTTP handlers.
//!
//! Authenticated callers can look up other users by username prefix to pick
//! transfer recipients. Search logic and exclusion of the current user (as
//! applicable) live in [`ficus_application::UserService`].

use axum::{
    extract::{Query, State},
    Json,
};
use ficus_application::dto::{PageResponse, UserSearchItem};
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

/// Query parameters for user search.
#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct UserSearchQuery {
    /// Username prefix to search for.
    pub query: String,
    /// Opaque pagination cursor from a previous page's `next_cursor`.
    pub cursor: Option<String>,
}

/// `GET /v1/users` — search users by username prefix.
///
/// # Auth
///
/// Requires Bearer JWT ([`AuthenticatedUser`]). The authenticated user id is
/// passed to the application layer so results can exclude or deprioritize self
/// according to product rules.
///
/// # Query
///
/// - `query` — required username prefix
/// - `cursor` — optional opaque pagination cursor
///
/// Page size uses [`AppState::default_page_size`].
///
/// # Errors
///
/// 400 on validation failures; 401 when unauthenticated.
#[utoipa::path(
    get,
    path = "/v1/users",
    params(UserSearchQuery),
    responses(
        (status = 200, description = "Search results", body = PageResponse<UserSearchItem>),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "users"
)]
pub async fn search_users(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<UserSearchQuery>,
) -> Result<Json<PageResponse<UserSearchItem>>, ApiError> {
    let page = state
        .users
        .search(
            &params.query,
            user.user_id,
            params.cursor.as_deref(),
            state.default_page_size,
        )
        .await?;

    let items = page
        .items
        .into_iter()
        .map(|u| UserSearchItem {
            user_id: u.id,
            username: u.username,
        })
        .collect();

    Ok(Json(PageResponse {
        items,
        next_cursor: page.next_cursor,
    }))
}
