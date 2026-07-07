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
    pub auth: Arc<AuthService>,
    pub users: Arc<UserService>,
    pub transfers: Arc<TransferService>,
    pub feed: Arc<FeedService>,
    pub tokens: Arc<dyn TokenService>,
    pub readiness: Option<Arc<dyn ReadinessCheck>>,
    pub login_rate_limit_per_min: u32,
    pub transfer_rate_limit_per_min: u32,
    pub default_page_size: u64,
}

impl AppState {
    /// Creates application state with the given services and rate-limit settings.
    pub fn new(
        auth: Arc<AuthService>,
        users: Arc<UserService>,
        transfers: Arc<TransferService>,
        feed: Arc<FeedService>,
        tokens: Arc<dyn TokenService>,
        login_rate_limit_per_min: u32,
        transfer_rate_limit_per_min: u32,
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
        }
    }

    /// Attaches an optional readiness probe for `/health/ready`.
    pub fn with_readiness(mut self, readiness: Arc<dyn ReadinessCheck>) -> Self {
        self.readiness = Some(readiness);
        self
    }
}
