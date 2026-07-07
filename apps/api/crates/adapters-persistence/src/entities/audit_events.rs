use sea_orm::entity::prelude::*;
use serde_json::Value as JsonValue;

/// Append-only audit event log entry.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "audit_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub event_type: String,
    pub actor_user_id: Option<Uuid>,
    pub transfer_id: Option<Uuid>,
    pub request_id: String,
    pub trace_id: String,
    pub metadata: JsonValue,
    pub occurred_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
