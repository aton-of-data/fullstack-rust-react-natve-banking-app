//! SeaORM migration entry point for the Ficus API database.

pub use sea_orm_migration::prelude::*;

mod m20250707_000001_create_initial_schema;

/// Registers all database migrations.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250707_000001_create_initial_schema::Migration)]
    }
}
