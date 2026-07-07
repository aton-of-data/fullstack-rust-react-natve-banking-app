use uuid::Uuid;

/// Transfer lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransferStatus {
    Completed,
    Declined,
}

/// A money transfer between two accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub id: Uuid,
    pub sender_account_id: Uuid,
    pub recipient_account_id: Uuid,
    pub amount_minor: i64,
    pub currency_code: String,
    pub description: Option<String>,
    pub status: TransferStatus,
}
