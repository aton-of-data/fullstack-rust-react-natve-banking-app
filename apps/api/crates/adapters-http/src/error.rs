//! HTTP error mapping for Axum handlers.
//!
//! Converts [`DomainError`] and transport failures into stable JSON
//! [`ErrorBody`] values. Client messages must never include passwords, JWTs,
//! raw idempotency keys, or SQL detail. Status mapping lives in
//! `ApiError::status` / private `domain_status`.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ficus_domain::errors::DomainError;
use serde::Serialize;
use thiserror::Error;

/// HTTP-layer API error with status mapping.
///
/// Prefer returning mapped domain errors for business failures. Use
/// [`ApiError::Internal`] only when the failure must not leak details.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Domain/business error — status derived from private `domain_status`.
    #[error("{0}")]
    Domain(#[from] DomainError),

    /// Missing or invalid bearer authentication.
    #[error("unauthorized")]
    Unauthorized,

    /// Transfer requested without an `Idempotency-Key` header (HTTP 400).
    #[error("missing idempotency key")]
    MissingIdempotencyKey,

    /// Login or transfer rate limit exceeded (HTTP 429).
    #[error("rate limit exceeded")]
    RateLimited,

    /// Request validation failure at the HTTP boundary (HTTP 400).
    #[error("validation error: {0}")]
    Validation(String),

    /// Unexpected internal failure; message is generic (HTTP 500).
    #[error("internal server error")]
    Internal,
}

/// Standard JSON error body returned to clients.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable message safe for clients.
    pub message: String,
}

impl ApiError {
    /// Maps this error to an HTTP status code.
    fn status(&self) -> StatusCode {
        match self {
            ApiError::Domain(err) => domain_status(err),
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::MissingIdempotencyKey => StatusCode::BAD_REQUEST,
            ApiError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Builds a client-safe JSON error body (no secrets or SQL).
    fn body(&self) -> ErrorBody {
        match self {
            ApiError::Domain(DomainError::InvalidCredentials) => ErrorBody {
                code: "INVALID_CREDENTIALS".into(),
                message: "Invalid username or password".into(),
            },
            ApiError::Domain(DomainError::InsufficientFunds) => ErrorBody {
                code: "INSUFFICIENT_FUNDS".into(),
                message: "Insufficient funds".into(),
            },
            ApiError::Domain(DomainError::SelfTransfer) => ErrorBody {
                code: "SELF_TRANSFER".into(),
                message: "Cannot transfer to yourself".into(),
            },
            ApiError::Domain(DomainError::RecipientNotFound) => ErrorBody {
                code: "RECIPIENT_NOT_FOUND".into(),
                message: "Recipient not found".into(),
            },
            ApiError::Domain(DomainError::UserNotFound) => ErrorBody {
                code: "USER_NOT_FOUND".into(),
                message: "User not found".into(),
            },
            ApiError::Domain(DomainError::AccountNotFound) => ErrorBody {
                code: "ACCOUNT_NOT_FOUND".into(),
                message: "Account not found".into(),
            },
            ApiError::Domain(DomainError::IdempotencyConflict) => ErrorBody {
                code: "IDEMPOTENCY_CONFLICT".into(),
                message: "Idempotency key reused with different payload".into(),
            },
            ApiError::Domain(DomainError::InvalidIdempotencyKey) => ErrorBody {
                code: "INVALID_IDEMPOTENCY_KEY".into(),
                message: "Invalid idempotency key".into(),
            },
            ApiError::Domain(DomainError::InvalidMoney(msg)) => ErrorBody {
                code: "INVALID_MONEY".into(),
                message: msg.clone(),
            },
            ApiError::Domain(DomainError::UnsupportedCurrency(msg)) => ErrorBody {
                code: "UNSUPPORTED_CURRENCY".into(),
                message: msg.clone(),
            },
            ApiError::Domain(DomainError::Validation(msg)) => ErrorBody {
                code: "VALIDATION_ERROR".into(),
                message: msg.clone(),
            },
            ApiError::Domain(_) => ErrorBody {
                code: "DOMAIN_ERROR".into(),
                message: "Request could not be completed".into(),
            },
            ApiError::Unauthorized => ErrorBody {
                code: "UNAUTHORIZED".into(),
                message: "Authentication required".into(),
            },
            ApiError::MissingIdempotencyKey => ErrorBody {
                code: "MISSING_IDEMPOTENCY_KEY".into(),
                message: "Idempotency-Key header is required".into(),
            },
            ApiError::RateLimited => ErrorBody {
                code: "RATE_LIMITED".into(),
                message: "Too many requests".into(),
            },
            ApiError::Validation(msg) => ErrorBody {
                code: "VALIDATION_ERROR".into(),
                message: msg.clone(),
            },
            ApiError::Internal => ErrorBody {
                code: "INTERNAL_ERROR".into(),
                message: "Internal server error".into(),
            },
        }
    }
}

/// Maps a domain error to the HTTP status used by the API contract.
fn domain_status(err: &DomainError) -> StatusCode {
    match err {
        DomainError::InvalidCredentials => StatusCode::UNAUTHORIZED,
        DomainError::UserNotFound
        | DomainError::RecipientNotFound
        | DomainError::AccountNotFound
        | DomainError::TransferNotFound => StatusCode::NOT_FOUND,
        DomainError::InsufficientFunds | DomainError::SelfTransfer => {
            StatusCode::UNPROCESSABLE_ENTITY
        }
        DomainError::IdempotencyConflict => StatusCode::CONFLICT,
        DomainError::InvalidIdempotencyKey
        | DomainError::InvalidMoney(_)
        | DomainError::UnsupportedCurrency(_)
        | DomainError::Validation(_) => StatusCode::BAD_REQUEST,
        DomainError::UnbalancedLedger | DomainError::NegativeBalance => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        (status, Json(self.body())).into_response()
    }
}
