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
    /// Opaque pagination cursor.
    pub cursor: Option<String>,
}

/// Searches users by username prefix.
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
