//! Account balance and ledger HTTP handlers.
//!
//! Authenticated routes under `/v1/accounts/me/*` expose the caller's current
//! balance and paginated double-entry ledger history. Handlers read through
//! [`ficus_application::UserService`]; they do not mutate balances.

use axum::{
    extract::{Query, State},
    Json,
};
use ficus_application::dto::{BalanceResponse, LedgerItemResponse, PageResponse};
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::AuthenticatedUser;
use crate::state::AppState;

/// Query parameters for paginated ledger history.
#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct LedgerQuery {
    /// Opaque pagination cursor from a previous page's `next_cursor`.
    pub cursor: Option<String>,
}

/// `GET /v1/accounts/me/balance` — current balance for the authenticated user.
///
/// # Auth
///
/// Requires Bearer JWT ([`AuthenticatedUser`]). Returns the caller's account
/// only; there is no path parameter for another user's balance.
///
/// # Errors
///
/// 401 unauthenticated; 404 when no account exists for the user.
#[utoipa::path(
    get,
    path = "/v1/accounts/me/balance",
    responses(
        (status = 200, description = "Account balance", body = BalanceResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 404, description = "Account not found", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "accounts"
)]
pub async fn get_balance(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<BalanceResponse>, ApiError> {
    let balance = state.users.get_balance(user.user_id).await?;
    Ok(Json(balance.into()))
}

/// `GET /v1/accounts/me/ledger` — paginated ledger entries for the caller.
///
/// # Auth
///
/// Requires Bearer JWT. Page size comes from [`AppState::default_page_size`].
/// Pass `cursor` from a prior response to continue.
///
/// # Errors
///
/// 401 unauthenticated; 404 when the account is missing.
#[utoipa::path(
    get,
    path = "/v1/accounts/me/ledger",
    params(LedgerQuery),
    responses(
        (status = 200, description = "Ledger page", body = PageResponse<LedgerItemResponse>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 404, description = "Account not found", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "accounts"
)]
pub async fn get_ledger(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<LedgerQuery>,
) -> Result<Json<PageResponse<LedgerItemResponse>>, ApiError> {
    let page = state
        .users
        .get_ledger(
            user.user_id,
            params.cursor.as_deref(),
            state.default_page_size,
        )
        .await?;

    let items = page.items.into_iter().map(Into::into).collect();
    Ok(Json(PageResponse {
        items,
        next_cursor: page.next_cursor,
    }))
}
