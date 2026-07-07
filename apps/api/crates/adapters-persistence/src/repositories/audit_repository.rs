use async_trait::async_trait;
use ficus_application::ports::AuditRepository;
use ficus_domain::audit::AuditEventDraft;
use ficus_domain::errors::DomainError;
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::entities::audit_events::Entity as AuditEvent;
use crate::error::map_db_err;
use crate::mapper::audit_draft_to_active;

/// SeaORM-backed append-only audit repository.
pub struct PostgresAuditRepository {
    db: DatabaseConnection,
}

impl PostgresAuditRepository {
    /// Creates a repository backed by the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn append(&self, event: AuditEventDraft) -> Result<(), DomainError> {
        let active = audit_draft_to_active(event);
        AuditEvent::insert(active)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}
