//! OpenAPI aggregation for Swagger UI (development/test only).
//!
//! [`ApiDoc`] is generated from `#[utoipa::path]` annotations on handlers and
//! DTO schemas. Runtime behavior is defined by handler code; this module only
//! documents the HTTP contract.

use utoipa::OpenApi;

use crate::handlers::{
    accounts::LedgerQuery, feed::FeedQuery, health::HealthResponse, users::UserSearchQuery,
};
use ficus_application::dto::{
    BalanceResponse, FeedItemResponse, LedgerItemResponse, LoginRequest, LoginResponse, MeResponse,
    PageResponse, TransferRequest, TransferResponse, UserSearchItem,
};

/// OpenAPI document for the Ficus HTTP API.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::login,
        crate::handlers::auth::logout,
        crate::handlers::auth::me,
        crate::handlers::users::search_users,
        crate::handlers::accounts::get_balance,
        crate::handlers::accounts::get_ledger,
        crate::handlers::transfers::create_transfer,
        crate::handlers::feed::list_feed,
        crate::handlers::feed::stream_feed,
        crate::handlers::health::live,
        crate::handlers::health::ready,
    ),
    components(schemas(
        LoginRequest,
        LoginResponse,
        MeResponse,
        UserSearchItem,
        UserSearchQuery,
        BalanceResponse,
        LedgerItemResponse,
        LedgerQuery,
        TransferRequest,
        TransferResponse,
        FeedItemResponse,
        FeedQuery,
        PageResponse<FeedItemResponse>,
        PageResponse<LedgerItemResponse>,
        PageResponse<UserSearchItem>,
        HealthResponse,
        crate::error::ErrorBody,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication"),
        (name = "users", description = "User search"),
        (name = "accounts", description = "Balances and ledger"),
        (name = "transfers", description = "Money transfers"),
        (name = "feed", description = "Global transaction feed"),
        (name = "health", description = "Health probes"),
    ),
    info(
        title = "Ficus API",
        version = "1.0.0",
        description = "Money transfer and balance tracking API"
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
