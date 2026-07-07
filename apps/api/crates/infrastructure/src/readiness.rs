use async_trait::async_trait;
use ficus_adapters_http::ReadinessCheck;
use sea_orm::DatabaseConnection;

/// Database readiness probe.
pub struct DbReadiness {
    db: DatabaseConnection,
}

impl DbReadiness {
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
