use std::sync::Arc;

use axum::Router;
use ficus_adapters_http::{create_router, AppState};
use ficus_adapters_persistence::{
    InMemoryFeedBroadcaster, PostgresAccountRepository, PostgresAuditRepository,
    PostgresIdempotencyRepository, PostgresTransferExecutor, PostgresTransferRepository,
    PostgresUserRepository,
};
use ficus_application::ports::{
    AccountRepository, AuditRepository, FeedBroadcaster, IdempotencyRepository, PasswordHasher,
    TokenService, TransferExecutor, TransferRepository, UserRepository,
};
use ficus_application::{AuthService, FeedService, TransferService, UserService};
use sea_orm::Database;

use crate::auth::{Argon2PasswordHasher, JwtTokenService};
use crate::config::AppConfig;
use crate::readiness::DbReadiness;

/// Builds the wired Axum application.
pub async fn build_app(config: &AppConfig) -> Result<Router, String> {
    let db = Database::connect(&config.database_url)
        .await
        .map_err(|e| format!("database connect failed: {e}"))?;

    let users: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(db.clone()));
    let accounts: Arc<dyn AccountRepository> = Arc::new(PostgresAccountRepository::new(db.clone()));
    let transfers_repo: Arc<dyn TransferRepository> =
        Arc::new(PostgresTransferRepository::new(db.clone()));
    let idempotency: Arc<dyn IdempotencyRepository> =
        Arc::new(PostgresIdempotencyRepository::new(db.clone()));
    let audit: Arc<dyn AuditRepository> = Arc::new(PostgresAuditRepository::new(db.clone()));
    let executor: Arc<dyn TransferExecutor> = Arc::new(PostgresTransferExecutor::new(db.clone()));
    let feed_broadcaster = InMemoryFeedBroadcaster::new(256).with_pg_notify(db.clone());
    feed_broadcaster
        .spawn_pg_listener()
        .await
        .map_err(|e| e.to_string())?;
    let feed: Arc<dyn FeedBroadcaster> = Arc::new(feed_broadcaster);

    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher);
    let tokens: Arc<dyn TokenService> = Arc::new(JwtTokenService::new(
        config.jwt_secret.clone(),
        config.jwt_expiry_secs,
    ));

    let auth = Arc::new(AuthService::new(
        users.clone(),
        hasher,
        tokens.clone(),
        audit.clone(),
    ));
    let users_svc = Arc::new(UserService::new(users, accounts.clone()));
    let transfer_svc = Arc::new(TransferService::new(executor, idempotency, feed.clone()));
    let feed_svc = Arc::new(FeedService::new(transfers_repo, feed));

    let state = AppState::new(
        auth,
        users_svc,
        transfer_svc,
        feed_svc,
        tokens,
        config.login_rate_limit_per_min,
        config.transfer_rate_limit_per_min,
    )
    .with_readiness(Arc::new(DbReadiness::new(db)));

    Ok(create_router(state))
}
