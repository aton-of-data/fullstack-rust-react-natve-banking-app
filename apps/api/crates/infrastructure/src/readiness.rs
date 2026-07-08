//! Database readiness probe for HTTP health/ready endpoints.
//!
//! Implements [`ficus_adapters_http::ReadinessCheck`] by pinging the SeaORM
//! connection so orchestrators can distinguish "process up" from "DB reachable".

use async_trait::async_trait;
use ficus_adapters_http::ReadinessCheck;
use sea_orm::DatabaseConnection;

/// Database readiness probe.
pub struct DbReadiness {
    db: DatabaseConnection,
}

impl DbReadiness {
    /// Wraps an existing SeaORM connection for readiness checks.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ReadinessCheck for DbReadiness {
    async fn is_ready(&self) -> bool {
        self.db.ping().await.is_ok()
    }
}
