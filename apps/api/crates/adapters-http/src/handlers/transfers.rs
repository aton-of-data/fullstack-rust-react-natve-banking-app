//! Money transfer HTTP handlers.
//!
//! Routes under `/v1/transfers` accept authenticated transfer requests and
//! delegate execution to [`ficus_application::TransferService`]. This module
//! does not implement ledger writes, balance locks, or idempotency storage —
//! it only enforces transport requirements (JWT, `Idempotency-Key` header)
//! and maps outcomes to HTTP responses.
//!
//! # OpenAPI
//!
//! Handler signatures are annotated with `#[utoipa::path(...)]`. Those
//! annotations feed [`crate::ApiDoc`], which is served as Swagger UI in
//! development/test environments. Runtime behavior is defined by the handler
//! bodies; OpenAPI documents the contract for clients.

use axum::{extract::State, http::HeaderMap, Json};
use ficus_application::dto::{TransferRequest, TransferResponse};
use ficus_domain::transfer::TransferStatus;

use crate::error::ApiError;
use crate::metrics::record_transfer;
use crate::middleware::{AuthenticatedUser, RequestContext};
use crate::state::AppState;

/// Canonical HTTP header name for transfer idempotency (`Idempotency-Key`).
///
/// Matching is case-insensitive via [`HeaderMap`]; the lowercase form below is
/// the lookup key Axum/hyper normalizes to.
const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";

/// `POST /v1/transfers` — create a money transfer (idempotent).
///
/// # Authentication
///
/// Requires a valid Bearer JWT. The route is mounted behind
/// [`crate::require_auth`], and [`AuthenticatedUser`] is extracted from the
/// request extensions. The authenticated user is always the **sender**;
/// clients cannot impersonate another sender via the body.
///
/// # Required headers
///
/// | Header | Required | Purpose |
/// | ------ | -------- | ------- |
/// | `Authorization: Bearer <jwt>` | yes | Identifies the sender |
/// | `Idempotency-Key` | yes | Client-supplied key for safe retries; missing/empty → [`ApiError::MissingIdempotencyKey`] (HTTP 400) |
/// | `X-Request-Id` / `X-Trace-Id` | optional | Propagated via [`RequestContext`] into application/audit |
///
/// # Request body
///
/// JSON [`TransferRequest`]:
/// - `recipient_username` — target user (must exist; self-transfer rejected in domain)
/// - `amount_minor` — positive integer minor units as a string
/// - `currency` — supported currency code
/// - `description` — optional note
///
/// Body size is capped globally by [`crate::MAX_BODY_BYTES`].
///
/// # Status mapping
///
/// Application/`DomainError` outcomes become [`ApiError`] and then JSON
/// [`crate::ErrorBody`] via [`IntoResponse`](axum::response::IntoResponse):
///
/// | Outcome | Typical HTTP status |
/// | ------- | ------------------- |
/// | Success (completed or declined record returned) | 200 + [`TransferResponse`] |
/// | Missing idempotency key | 400 `MISSING_IDEMPOTENCY_KEY` |
/// | Validation / invalid money / unsupported currency | 400 |
/// | Unauthorized / bad JWT | 401 |
/// | Idempotency key reused with different payload | 409 |
/// | Insufficient funds / self-transfer | 422 |
/// | Rate limited (per-user transfer limiter) | 429 |
///
/// Declined transfers that the application returns as a successful record
/// (for example insufficient funds handled as a declined result) are still
/// HTTP 200 with status reflected in the response body; errors that abort
/// before a record is returned map through [`ApiError`].
///
/// # Sensitive data
///
/// Do **not** log passwords, JWTs, or raw `Authorization` headers. Amounts,
/// currencies, usernames, transfer ids, and correlation ids are acceptable for
/// metrics and structured tracing. Metrics here record only coarse outcomes
/// (`completed`, `declined`, `error`) via [`record_transfer`].
///
/// # OpenAPI / utoipa
///
/// The `#[utoipa::path]` attribute below is registered on [`crate::ApiDoc`]
/// under the `transfers` tag with `bearer_auth` security. Keep the annotation
/// response list aligned with [`ApiError`] mapping when adding new codes.
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
