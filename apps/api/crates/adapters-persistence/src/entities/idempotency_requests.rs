use sea_orm::entity::prelude::*;

/// Stored idempotency response for safe transfer retries.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "idempotency_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sender_user_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub idempotency_key: String,
    pub fingerprint: String,
    #[sea_orm(column_type = "Text")]
    pub response_body: String,
    pub status_code: i16,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::SenderUserId",
        to = "super::users::Column::Id"
    )]
    Sender,
}

impl ActiveModelBehavior for ActiveModel {}
