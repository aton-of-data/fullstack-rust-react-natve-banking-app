use axum::{extract::State, http::HeaderMap, Json};
use ficus_application::dto::{TransferRequest, TransferResponse};
use ficus_domain::transfer::TransferStatus;

use crate::error::ApiError;
use crate::metrics::record_transfer;
use crate::middleware::{AuthenticatedUser, RequestContext};
use crate::state::AppState;

const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";

/// Creates a money transfer with idempotency support.
#[utoipa::path(
    post,
    path = "/v1/transfers",
    request_body = TransferRequest,
    responses(
        (status = 200, description = "Transfer result", body = TransferResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 409, description = "Idempotency conflict", body = crate::error::ErrorBody),
        (status = 422, description = "Business rule violation", body = crate::error::ErrorBody),
        (status = 429, description = "Rate limited", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "transfers"
)]
pub async fn create_transfer(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ctx: axum::Extension<RequestContext>,
    headers: HeaderMap,
    Json(body): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, ApiError> {
    let idempotency_key = headers
        .get(IDEMPOTENCY_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(ApiError::MissingIdempotencyKey)?;

    let record = match state
        .transfers
        .transfer(
            user.user_id,
            &body.recipient_username,
            &body.amount_minor,
            &body.currency,
            body.description.as_deref(),
            idempotency_key,
            &ctx.request_id,
            &ctx.trace_id,
        )
        .await
    {
        Ok(record) => {
            let outcome = match record.status {
                TransferStatus::Completed => "completed",
                TransferStatus::Declined => "declined",
            };
            record_transfer(outcome);
            record
        }
        Err(err) => {
            record_transfer("error");
            return Err(err.into());
        }
    };

    let balance = state.users.get_balance(user.user_id).await?;
    let mut response = TransferResponse::from(record);
    response.sender_balance_minor = balance.balance_minor.to_string();
    Ok(Json(response))
}
