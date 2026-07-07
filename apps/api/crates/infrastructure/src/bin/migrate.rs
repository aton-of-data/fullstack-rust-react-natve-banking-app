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
