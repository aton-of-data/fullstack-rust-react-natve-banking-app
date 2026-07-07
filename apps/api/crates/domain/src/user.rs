use uuid::Uuid;

/// Domain user aggregate root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

impl User {
    /// Creates a user with the given id and username.
    pub fn new(id: Uuid, username: impl Into<String>) -> Self {
        Self {
            id,
            username: username.into(),
        }
    }
}
