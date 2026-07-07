//! Wired application services and optional HTTP server for integration tests.

use std::net::SocketAddr;
use std::sync::Arc;

use ficus_adapters_persistence::{
    InMemoryFeedBroadcaster, PostgresAccountRepository, PostgresAuditRepository,
    PostgresIdempotencyRepository, PostgresTransferExecutor, PostgresTransferRepository,
    PostgresUserRepository,
};
use ficus_application::ports::{AccountRepository, TransferExecutor, TransferRepository};
use ficus_application::ports::{
    AuditRepository, FeedBroadcaster, IdempotencyRepository, PasswordHasher, TokenService,
    UserRepository,
};
use ficus_application::{AuthService, UserService};
use ficus_application::{FeedService, TransferService};
use ficus_infrastructure::auth::{Argon2PasswordHasher, JwtTokenService};
use ficus_infrastructure::config::AppConfig;
use ficus_infrastructure::wiring::build_app;
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::seed::{seed_test_users, TestUsers};

/// A seeded user with account metadata.
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub account_id: uuid::Uuid,
    pub initial_balance_minor: i64,
}

/// Wired services for direct use-case testing.
pub struct WiredServices {
    pub db: DatabaseConnection,
    pub users: TestUsers,
    pub transfer_service: Arc<TransferService>,
    pub accounts: Arc<dyn AccountRepository>,
    pub transfers: Arc<dyn TransferRepository>,
    pub executor: Arc<dyn TransferExecutor>,
}

/// Integration test harness with optional HTTP server.
pub struct TestApp {
    pub db: DatabaseConnection,
    pub users: TestUsers,
    pub transfer_service: Arc<TransferService>,
    pub accounts: Arc<dyn AccountRepository>,
    pub transfers: Arc<dyn TransferRepository>,
    pub executor: Arc<dyn TransferExecutor>,
    pub base_url: Option<String>,
    server_handle: Option<JoinHandle<()>>,
}

impl TestApp {
    /// Returns the HTTP base URL when a server was started.
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

/// Builder for a fully wired test application.
pub struct TestAppBuilder {
    database_url: String,
    db: Option<DatabaseConnection>,
    users: Option<TestUsers>,
    start_server: bool,
    login_rate_limit_per_min: Option<u32>,
    transfer_rate_limit_per_min: Option<u32>,
    environment: Option<String>,
    metrics_auth_token: Option<String>,
}

impl TestAppBuilder {
    /// Creates a builder for the given database URL.
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            db: None,
            users: None,
            start_server: false,
            login_rate_limit_per_min: None,
            transfer_rate_limit_per_min: None,
            environment: None,
            metrics_auth_token: None,
        }
    }

    /// Uses an existing database connection instead of connecting again.
    pub fn with_db(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    /// Uses pre-seeded users instead of running the default seed.
    pub fn with_users(mut self, users: TestUsers) -> Self {
        self.users = Some(users);
        self
    }

    /// Starts the HTTP API on an ephemeral local port.
    pub fn with_http_server(mut self) -> Self {
        self.start_server = true;
        self
    }

    /// Overrides login rate limit for deterministic HTTP 429 tests.
    pub fn with_login_rate_limit(mut self, per_min: u32) -> Self {
        self.login_rate_limit_per_min = Some(per_min);
        self
    }

    /// Overrides transfer rate limit for deterministic HTTP 429 tests.
    pub fn with_transfer_rate_limit(mut self, per_min: u32) -> Self {
        self.transfer_rate_limit_per_min = Some(per_min);
        self
    }

    /// Overrides deployment environment for security contract tests.
    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    /// Sets the bearer token required for `/metrics` when auth is enforced.
    pub fn with_metrics_auth_token(mut self, token: impl Into<String>) -> Self {
        self.metrics_auth_token = Some(token.into());
        self
    }

    /// Builds wired services and optionally starts HTTP.
    pub async fn build(self) -> Result<TestApp, String> {
        let db = if let Some(db) = self.db {
            db
        } else {
            sea_orm::Database::connect(&self.database_url)
                .await
                .map_err(|e| format!("database connect failed: {e}"))?
        };

        let users = if let Some(users) = self.users {
            users
        } else {
            seed_test_users(&db)
                .await
                .map_err(|e| format!("seed failed: {e}"))?
        };

        let wired = wire_services(db.clone(), users.clone()).await?;

        let (base_url, server_handle) = if self.start_server {
            let config = test_app_config(
                &self.database_url,
                self.login_rate_limit_per_min,
                self.transfer_rate_limit_per_min,
                self.environment.as_deref(),
                self.metrics_auth_token.as_deref(),
            );
            let (url, handle) = spawn_http_server(&config).await?;
            (Some(url), Some(handle))
        } else {
            (None, None)
        };

        Ok(TestApp {
            db,
            users: wired.users,
            transfer_service: wired.transfer_service,
            accounts: wired.accounts,
            transfers: wired.transfers,
            executor: wired.executor,
            base_url,
            server_handle,
        })
    }
}

/// Wires repositories and use cases without starting HTTP.
pub async fn wire_services(
    db: DatabaseConnection,
    users: TestUsers,
) -> Result<WiredServices, String> {
    let users_repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(db.clone()));
    let accounts: Arc<dyn AccountRepository> = Arc::new(PostgresAccountRepository::new(db.clone()));
    let transfers_repo: Arc<dyn TransferRepository> =
        Arc::new(PostgresTransferRepository::new(db.clone()));
    let idempotency: Arc<dyn IdempotencyRepository> =
        Arc::new(PostgresIdempotencyRepository::new(db.clone()));
    let audit: Arc<dyn AuditRepository> = Arc::new(PostgresAuditRepository::new(db.clone()));
    let executor: Arc<dyn TransferExecutor> = Arc::new(PostgresTransferExecutor::new(db.clone()));
    let feed: Arc<dyn FeedBroadcaster> = Arc::new(InMemoryFeedBroadcaster::new(256));

    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher);
    let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(test_jwt_secret(), 3600));

    let auth = Arc::new(AuthService::new(
        users_repo.clone(),
        hasher,
        tokens.clone(),
        audit.clone(),
    ));
    let users_svc = Arc::new(UserService::new(users_repo, accounts.clone()));
    let transfer_service = Arc::new(TransferService::new(
        executor.clone(),
        idempotency,
        feed.clone(),
    ));
    let _feed_svc = Arc::new(FeedService::new(transfers_repo.clone(), feed));
    let _ = auth;
    let _ = users_svc;

    Ok(WiredServices {
        db,
        users,
        transfer_service,
        accounts,
        transfers: transfers_repo,
        executor,
    })
}

fn test_jwt_secret() -> String {
    "integration-test-jwt-secret-32chars-min".to_string()
}

fn test_app_config(
    database_url: &str,
    login_rate_limit_per_min: Option<u32>,
    transfer_rate_limit_per_min: Option<u32>,
    environment: Option<&str>,
    metrics_auth_token: Option<&str>,
) -> AppConfig {
    AppConfig {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: database_url.to_string(),
        migration_database_url: database_url.to_string(),
        jwt_secret: test_jwt_secret(),
        jwt_expiry_secs: 3600,
        environment: environment.unwrap_or("test").to_string(),
        cors_origins: vec!["http://localhost".into()],
        login_rate_limit_per_min: login_rate_limit_per_min.unwrap_or(10_000),
        transfer_rate_limit_per_min: transfer_rate_limit_per_min.unwrap_or(10_000),
        otel_endpoint: None,
        otel_service_name: "ficus-api-test".into(),
        metrics_auth_token: metrics_auth_token.map(str::to_string),
        trust_proxy_headers: false,
    }
}

async fn spawn_http_server(config: &AppConfig) -> Result<(String, JoinHandle<()>), String> {
    let app = build_app(config).await?;
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .map_err(|e| format!("bind failed: {e}"))?;
    let addr = listener
        .local_addr()
        .map_err(|e| format!("local_addr failed: {e}"))?;
    let base_url = format!("http://{addr}");

    let handle = tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            tracing::error!(error = %err, "test http server stopped");
        }
    });

    Ok((base_url, handle))
}
