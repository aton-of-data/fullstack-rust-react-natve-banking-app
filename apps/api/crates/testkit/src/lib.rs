//! Ficus integration testkit — Postgres testcontainers, migrations, and wired services.

mod app;
mod assertions;
mod http;
mod seed;

pub use app::{wire_services, TestApp, TestAppBuilder, TestUser, WiredServices};
pub use assertions::{
    count_all_transfers, count_audit_events, count_completed_transfers, count_idempotency_records,
    count_ledger_entries, ledger_derived_balance, negative_balances, orphan_ledger_entries,
    reconcile_all_accounts, signed_ledger_amount, total_balance_minor, ReconciliationMismatch,
};
pub use http::{
    http_client, http_create_transfer, http_create_transfer_raw, http_get_balance, http_get_feed,
    http_get_ledger, http_get_me, http_get_metrics, http_login, http_login_response, http_logout,
    HttpJsonResponse, HttpTransferParams,
};
pub use seed::{seed_test_users, set_account_balance, TestUsers, SYSTEM_ACCOUNT_ID};

use ficus_adapters_persistence::Migrator;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;
use tokio::sync::Mutex;

static MIGRATION_LOCK: Mutex<()> = Mutex::const_new(());
static TEST_ISOLATION_LOCK: Mutex<()> = Mutex::const_new(());

/// Running PostgreSQL testcontainer with connection URL.
pub struct PostgresFixture {
    pub database_url: String,
    _container: Option<ContainerAsync<Postgres>>,
}

impl PostgresFixture {
    fn external(database_url: String) -> Self {
        Self {
            database_url,
            _container: None,
        }
    }
}

/// Starts a disposable PostgreSQL instance via testcontainers.
///
/// When `TEST_DATABASE_URL` is set, uses that database instead (no container).
pub async fn start_postgres() -> PostgresFixture {
    if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        return PostgresFixture::external(url);
    }

    let container = Postgres::default().start().await.unwrap_or_else(|err| {
        panic!(
            "failed to start postgres container ({err}). \
             Start Docker or set TEST_DATABASE_URL to an existing Postgres instance."
        )
    });

    let host = container.get_host().await.expect("postgres host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("postgres port");

    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    PostgresFixture {
        database_url,
        _container: Some(container),
    }
}

/// Connects and applies all SeaORM migrations (serialized to avoid races).
pub async fn run_migrations(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let _guard = MIGRATION_LOCK.lock().await;
    let db = Database::connect(database_url).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

/// Truncates application tables for isolated integration tests.
pub async fn reset_test_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let _guard = TEST_ISOLATION_LOCK.lock().await;
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "TRUNCATE TABLE audit_events, idempotency_requests, ledger_entries, transfers, account_balances, accounts, users RESTART IDENTITY CASCADE",
    ))
    .await?;
    Ok(())
}

/// Starts Postgres, runs migrations, resets data, and seeds default users.
pub async fn setup_isolated_test_db(
) -> Result<(PostgresFixture, DatabaseConnection, TestUsers), String> {
    let pg = start_postgres().await;
    let db = run_migrations(&pg.database_url)
        .await
        .map_err(|e| format!("migration failed: {e}"))?;
    reset_test_database(&db)
        .await
        .map_err(|e| format!("reset failed: {e}"))?;
    let users = seed_test_users(&db)
        .await
        .map_err(|e| format!("seed failed: {e}"))?;
    Ok((pg, db, users))
}

/// Convenience transfer invocation with tracing metadata.
pub async fn execute_transfer(
    app: &TestApp,
    sender_user_id: uuid::Uuid,
    recipient_username: &str,
    amount_minor: i64,
    idempotency_key: &str,
) -> Result<ficus_application::ports::TransferRecord, ficus_domain::errors::DomainError> {
    app.transfer_service
        .transfer(
            sender_user_id,
            recipient_username,
            &amount_minor.to_string(),
            "USD",
            None,
            idempotency_key,
            &format!("req-{idempotency_key}"),
            &format!("trace-{idempotency_key}"),
        )
        .await
}
