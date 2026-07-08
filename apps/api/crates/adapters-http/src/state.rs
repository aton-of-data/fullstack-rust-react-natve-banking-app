//! Shared Axum application state.
//!
//! Built by the infrastructure composition root and injected into handlers.
//! Contains application services and runtime knobs only — no SeaORM types.

use std::sync::Arc;

use async_trait::async_trait;
use ficus_application::ports::TokenService;
use ficus_application::{AuthService, FeedService, TransferService, UserService};

/// Optional readiness probe wired by the composition root.
#[async_trait]
pub trait ReadinessCheck: Send + Sync {
    /// Returns true when dependencies (e.g. database) are available.
    async fn is_ready(&self) -> bool;
}

/// Shared application services and infrastructure handles for HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    /// Authentication use cases.
    pub auth: Arc<AuthService>,
    /// User search / balance / ledger use cases.
    pub users: Arc<UserService>,
    /// Transfer orchestration use cases.
    pub transfers: Arc<TransferService>,
    /// Feed list and subscribe use cases.
    pub feed: Arc<FeedService>,
    /// JWT verification used by auth middleware.
    pub tokens: Arc<dyn TokenService>,
    /// Optional readiness probe for `/health/ready`.
    pub readiness: Option<Arc<dyn ReadinessCheck>>,
    /// Login attempts allowed per minute per client key.
    pub login_rate_limit_per_min: u32,
    /// Transfer attempts allowed per minute per authenticated user.
    pub transfer_rate_limit_per_min: u32,
    /// Default pagination page size for list endpoints.
    pub default_page_size: u64,
    /// Allowed CORS origins.
    pub cors_origins: Vec<String>,
    /// Deployment environment name (`development`, `test`, `production`, …).
    pub environment: String,
    /// Bearer token required for `/metrics` when set.
    pub metrics_auth_token: Option<String>,
    /// When true, honor `X-Forwarded-For` for login rate-limit keys.
    pub trust_proxy_headers: bool,
}

impl AppState {
    /// Creates application state with the given services and rate-limit settings.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth: Arc<AuthService>,
        users: Arc<UserService>,
        transfers: Arc<TransferService>,
        feed: Arc<FeedService>,
        tokens: Arc<dyn TokenService>,
        login_rate_limit_per_min: u32,
        transfer_rate_limit_per_min: u32,
        cors_origins: Vec<String>,
        environment: String,
        metrics_auth_token: Option<String>,
        trust_proxy_headers: bool,
    ) -> Self {
        Self {
            auth,
            users,
            transfers,
            feed,
            tokens,
            readiness: None,
            login_rate_limit_per_min,
            transfer_rate_limit_per_min,
            default_page_size: 20,
            cors_origins,
            environment,
            metrics_auth_token,
            trust_proxy_headers,
        }
    }

    /// Attaches an optional readiness probe for `/health/ready`.
    pub fn with_readiness(mut self, readiness: Arc<dyn ReadinessCheck>) -> Self {
        self.readiness = Some(readiness);
        self
    }
}
