//! Ficus integration testkit — Postgres fixtures, seed helpers, and assertions.
//!
//! Provides disposable (or `TEST_DATABASE_URL`) Postgres, SeaORM migrations,
//! truncate/seed isolation, wired transfer-service harnesses, HTTP helpers,
//! and ledger/balance assertion utilities used by financial integration tests.
//!
//! # Architectural role
//!
//! Test-only support crate above `infrastructure`. It wires the same ports as
//! production for in-process tests, and can start an HTTP server via
//! [`TestAppBuilder`].
//!
//! # Isolation helpers
//!
//! - [`setup_isolated_test_db`] — start Postgres, migrate, truncate, seed
//!   default users under an advisory lock.
//! - [`PostgresFixture`] — running container or external URL.
//! - [`reset_test_database`] — truncate application tables (CASCADE).
//!
//! **Warning:** truncate/seed helpers **bypass production transfer flows**.
//! They insert users/accounts and system funding (or direct balance updates)
//! outside the live API path. Use them to establish fixtures only; assert
//! money movement through [`execute_transfer`] / HTTP helpers when proving
//! production invariants.
//!
//! # Neighboring crates
//!
//! Depends on infrastructure wiring/auth and persistence repositories. Does
//! not redefine domain ledger rules — it checks them after exercising services.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::bare_urls)]

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
    http_get_ledger, http_get_me, http_get_metrics, http_get_metrics_with_auth, http_login,
    http_login_response, http_logout, HttpJsonResponse, HttpTransferParams,
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

/// Advisory lock key serializing truncate/seed across parallel test binaries.
const TEST_ISOLATION_ADVISORY_LOCK: i64 = 0x4649_4355;

/// Running PostgreSQL testcontainer with connection URL.
///
/// Holds the container handle (when started via testcontainers) so the instance
/// stays alive for the duration of the test that owns this fixture.
pub struct PostgresFixture {
    /// Postgres connection URL for SeaORM / Axum wiring.
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
///
/// **Bypasses production transfer flows** — destroys rows via `TRUNCATE ...
/// CASCADE` under an advisory lock. Prefer after migrate, before seed.
pub async fn reset_test_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    reset_test_database_locked(db).await
}

async fn reset_test_database_locked(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let _guard = TEST_ISOLATION_LOCK.lock().await;
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!("SELECT pg_advisory_lock({TEST_ISOLATION_ADVISORY_LOCK})"),
    ))
    .await?;

    let truncate = db
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "TRUNCATE TABLE audit_events, idempotency_requests, ledger_entries, transfers, account_balances, accounts, users RESTART IDENTITY CASCADE",
        ))
        .await;

    let _ = db
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("SELECT pg_advisory_unlock({TEST_ISOLATION_ADVISORY_LOCK})"),
        ))
        .await;

    truncate?;
    Ok(())
}

/// Starts Postgres, runs migrations, resets data, and seeds default users.
///
/// Truncate + seed **bypass production transfer APIs**; funding uses seed
/// helpers so tests start from a known balance snapshot.
pub async fn setup_isolated_test_db(
) -> Result<(PostgresFixture, DatabaseConnection, TestUsers), String> {
    let pg = start_postgres().await;
    let db = run_migrations(&pg.database_url)
        .await
        .map_err(|e| format!("migration failed: {e}"))?;

    let _guard = TEST_ISOLATION_LOCK.lock().await;
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!("SELECT pg_advisory_lock({TEST_ISOLATION_ADVISORY_LOCK})"),
    ))
    .await
    .map_err(|e| format!("isolation lock failed: {e}"))?;

    let setup_result = async {
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "TRUNCATE TABLE audit_events, idempotency_requests, ledger_entries, transfers, account_balances, accounts, users RESTART IDENTITY CASCADE",
        ))
        .await
        .map_err(|e| format!("reset failed: {e}"))?;
        seed_test_users(&db)
            .await
            .map_err(|e| format!("seed failed: {e}"))
    }
    .await;

    let _ = db
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("SELECT pg_advisory_unlock({TEST_ISOLATION_ADVISORY_LOCK})"),
        ))
        .await;

    let users = setup_result?;
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
