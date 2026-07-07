use sea_orm::entity::prelude::*;

/// Transfer aggregate persisted for feed and history.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "transfers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub sender_account_id: Uuid,
    pub recipient_account_id: Uuid,
    pub sender_user_id: Uuid,
    pub recipient_user_id: Uuid,
    pub amount_minor: i64,
    pub currency_code: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::accounts::Entity",
        from = "Column::SenderAccountId",
        to = "super::accounts::Column::Id"
    )]
    SenderAccount,
    #[sea_orm(
        belongs_to = "super::accounts::Entity",
        from = "Column::RecipientAccountId",
        to = "super::accounts::Column::Id"
    )]
    RecipientAccount,
    #[sea_orm(has_many = "super::ledger_entries::Entity")]
    LedgerEntries,
}

impl ActiveModelBehavior for ActiveModel {}
