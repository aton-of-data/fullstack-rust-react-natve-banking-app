use uuid::Uuid;

/// A financial account owned by a user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid,
    pub is_system: bool,
}

impl Account {
    /// Creates a regular user account.
    pub fn user_account(id: Uuid, user_id: Uuid) -> Self {
        Self {
            id,
            user_id,
            is_system: false,
        }
    }

    /// Creates the system funding account.
    pub fn system_funding(id: Uuid) -> Self {
        Self {
            id,
            user_id: Uuid::nil(),
            is_system: true,
        }
    }
}
