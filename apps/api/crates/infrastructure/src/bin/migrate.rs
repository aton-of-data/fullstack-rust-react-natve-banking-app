//! `migrate` binary — apply SeaORM schema migrations.
//!
//! Connects with `DATABASE_MIGRATION_URL` (or `DATABASE_URL`) and runs
//! `Migrator::up`. Safe to re-run; migrations are ordered and tracked by
//! SeaORM. Does not seed users or balances.

use ficus_adapters_persistence::Migrator;
use ficus_infrastructure::AppConfig;
use sea_orm_migration::MigratorTrait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env().map_err(|e| format!("config error: {e}"))?;
    let db = sea_orm::Database::connect(&config.migration_database_url).await?;
    Migrator::up(&db, None).await?;
    println!("migrations applied");
    Ok(())
}
