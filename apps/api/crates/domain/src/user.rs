//! User aggregate root.
//!
//! This module belongs to the domain layer. It identifies people who own
//! accounts and authenticate. Password hashes and JWT material live behind
//! application ports / infrastructure — never on this type.

use uuid::Uuid;

/// Domain user aggregate root.
///
/// Usernames are unique at the persistence layer. This type deliberately omits
/// credentials so domain logic cannot accidentally log or serialize secrets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Stable user identifier.
    pub id: Uuid,
    /// Unique public username.
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
