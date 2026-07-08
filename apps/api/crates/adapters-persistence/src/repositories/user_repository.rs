//! User lookup and username-prefix search repository.
//!
//! **Tables:** `users`.
//!
//! Returns application [`UserRecord`](ficus_application::ports::UserRecord) values
//! including `password_hash` for auth verification. Callers must never log or
//! serialize the hash to clients. This repository does not create users (seed
//! / migrate paths own inserts).

use async_trait::async_trait;
use ficus_application::ports::{Page, UserRecord, UserRepository};
use ficus_domain::errors::DomainError;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::entities::users::{self, Entity as User};
use crate::error::map_db_err;
use crate::mapper::user_to_record;

/// SeaORM-backed user repository (`users`).
pub struct PostgresUserRepository {
    db: DatabaseConnection,
}

impl PostgresUserRepository {
    /// Creates a repository backed by the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>, DomainError> {
        let row = User::find()
            .filter(users::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(row.map(user_to_record))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRecord>, DomainError> {
        let row = User::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(row.map(user_to_record))
    }

    async fn search_by_username(
        &self,
        query: &str,
        exclude_user_id: Uuid,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<Page<UserRecord>, DomainError> {
        let pattern = format!("{}%", query);
        let mut condition = Condition::all()
            .add(users::Column::Username.like(pattern))
            .add(users::Column::Id.ne(exclude_user_id));

        if let Some(cursor) = cursor {
            let (username, id) = cursor
                .split_once('|')
                .ok_or_else(|| DomainError::Validation("invalid cursor".into()))?;
            let id = Uuid::parse_str(id)
                .map_err(|_| DomainError::Validation("invalid cursor id".into()))?;
            condition = condition.add(
                Condition::any()
                    .add(users::Column::Username.gt(username))
                    .add(
                        Condition::all()
                            .add(users::Column::Username.eq(username))
                            .add(users::Column::Id.gt(id)),
                    ),
            );
        }

        let rows = User::find()
            .filter(condition)
            .order_by_asc(users::Column::Username)
            .order_by_asc(users::Column::Id)
            .limit(limit + 1)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;

        let has_more = rows.len() > limit as usize;
        let items: Vec<UserRecord> = rows
            .into_iter()
            .take(limit as usize)
            .map(user_to_record)
            .collect();

        let next_cursor = if has_more {
            items
                .last()
                .map(|user| format!("{}|{}", user.username, user.id))
        } else {
            None
        };

        Ok(Page { items, next_cursor })
    }
}
