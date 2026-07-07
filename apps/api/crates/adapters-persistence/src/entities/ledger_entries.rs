use sea_orm::entity::prelude::*;

/// Append-only ledger entry for double-entry bookkeeping.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "ledger_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub transfer_id: Uuid,
    pub amount_minor: i64,
    pub direction: String,
    pub currency_code: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::accounts::Entity",
        from = "Column::AccountId",
        to = "super::accounts::Column::Id"
    )]
    Account,
    #[sea_orm(
        belongs_to = "super::transfers::Entity",
        from = "Column::TransferId",
        to = "super::transfers::Column::Id"
    )]
    Transfer,
}

impl Related<super::transfers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transfer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
