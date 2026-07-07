use async_trait::async_trait;
use ficus_application::ports::{FeedItem, Page, TransferRecord, TransferRepository};
use ficus_domain::errors::DomainError;
use ficus_domain::transfer::TransferStatus;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::entities::transfers::{self, Entity as Transfer};
use crate::entities::users;
use crate::error::map_db_err;
use crate::mapper::{
    decode_cursor, encode_cursor, transfer_status_to_db, transfer_to_feed_item, transfer_to_record,
};

/// SeaORM-backed transfer repository.
pub struct PostgresTransferRepository {
    db: DatabaseConnection,
}

impl PostgresTransferRepository {
    /// Creates a repository backed by the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TransferRepository for PostgresTransferRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<TransferRecord>, DomainError> {
        let transfer = Transfer::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?;

        let Some(transfer) = transfer else {
            return Ok(None);
        };

        let sender = users::Entity::find_by_id(transfer.sender_user_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::UserNotFound)?;
        let recipient = users::Entity::find_by_id(transfer.recipient_user_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
            .ok_or(DomainError::UserNotFound)?;

        Ok(Some(transfer_to_record(
            transfer,
            sender.username,
            recipient.username,
        )?))
    }

    async fn list_feed(
        &self,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<FeedItem>, DomainError> {
        let mut condition = Condition::all()
            .add(transfers::Column::Status.eq(transfer_status_to_db(TransferStatus::Completed)));

        if let Some(cursor) = cursor {
            let (created_at, id) = decode_cursor(cursor)?;
            let ts: sea_orm::prelude::DateTimeWithTimeZone = created_at.into();
            condition = condition.add(
                Condition::any()
                    .add(transfers::Column::CreatedAt.lt(ts))
                    .add(
                        Condition::all()
                            .add(transfers::Column::CreatedAt.eq(ts))
                            .add(transfers::Column::Id.lt(id)),
                    ),
            );
        }

        let rows = Transfer::find()
            .filter(condition)
            .order_by_desc(transfers::Column::CreatedAt)
            .order_by_desc(transfers::Column::Id)
            .limit(limit + 1)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;

        let has_more = rows.len() > limit as usize;
        let mut items = Vec::with_capacity(rows.len().min(limit as usize));

        for transfer in rows.into_iter().take(limit as usize) {
            let sender = users::Entity::find_by_id(transfer.sender_user_id)
                .one(&self.db)
                .await
                .map_err(map_db_err)?
                .ok_or(DomainError::UserNotFound)?;
            let recipient = users::Entity::find_by_id(transfer.recipient_user_id)
                .one(&self.db)
                .await
                .map_err(map_db_err)?
                .ok_or(DomainError::UserNotFound)?;
            items.push(transfer_to_feed_item(
                transfer,
                sender.username,
                recipient.username,
            ));
        }

        let next_cursor = if has_more {
            items
                .last()
                .map(|item| encode_cursor(item.created_at, item.transfer_id))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }
}
