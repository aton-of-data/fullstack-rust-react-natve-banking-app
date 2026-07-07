use async_trait::async_trait;
use ficus_application::ports::{IdempotencyRecord, IdempotencyRepository};
use ficus_domain::errors::DomainError;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entities::idempotency_requests::{self, ActiveModel, Entity as IdempotencyRequest};
use crate::error::map_db_err;
use crate::mapper::idempotency_to_record;

/// SeaORM-backed idempotency repository.
pub struct PostgresIdempotencyRepository {
    db: DatabaseConnection,
}

impl PostgresIdempotencyRepository {
    /// Creates a repository backed by the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IdempotencyRepository for PostgresIdempotencyRepository {
    async fn find(
        &self,
        sender_user_id: Uuid,
        idempotency_key: &str,
    ) -> Result<Option<IdempotencyRecord>, DomainError> {
        let row = IdempotencyRequest::find()
            .filter(idempotency_requests::Column::SenderUserId.eq(sender_user_id))
            .filter(idempotency_requests::Column::IdempotencyKey.eq(idempotency_key))
            .one(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(row.map(idempotency_to_record))
    }

    async fn store(
        &self,
        sender_user_id: Uuid,
        idempotency_key: &str,
        fingerprint: &str,
        response_body: &str,
        status_code: u16,
    ) -> Result<(), DomainError> {
        let model = ActiveModel {
            sender_user_id: sea_orm::Set(sender_user_id),
            idempotency_key: sea_orm::Set(idempotency_key.to_string()),
            fingerprint: sea_orm::Set(fingerprint.to_string()),
            response_body: sea_orm::Set(response_body.to_string()),
            status_code: sea_orm::Set(status_code as i16),
            created_at: sea_orm::Set(chrono::Utc::now().into()),
        };

        IdempotencyRequest::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    idempotency_requests::Column::SenderUserId,
                    idempotency_requests::Column::IdempotencyKey,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;

        Ok(())
    }
}
