use std::sync::Arc;

use chrono::Utc;
use ficus_domain::audit::{AuditEventDraft, AuditEventType};
use ficus_domain::errors::DomainError;
use uuid::Uuid;

use crate::ports::{AuditRepository, PasswordHasher, TokenService, UserRepository};

/// Authentication use cases.
pub struct AuthService {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    audit: Arc<dyn AuditRepository>,
}

impl AuthService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
        audit: Arc<dyn AuditRepository>,
    ) -> Self {
        Self {
            users,
            hasher,
            tokens,
            audit,
        }
    }

    /// Authenticates user and returns JWT.
    pub async fn login(
        &self,
        username: &str,
        password: &str,
        request_id: &str,
        trace_id: &str,
    ) -> Result<(String, Uuid, String), DomainError> {
        let user = self
            .users
            .find_by_username(username)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        let valid = self.hasher.verify(password, &user.password_hash).await?;
        if !valid {
            self.audit
                .append(AuditEventDraft {
                    event_type: AuditEventType::LoginFailure,
                    actor_user_id: None,
                    transfer_id: None,
                    request_id: request_id.to_string(),
                    trace_id: trace_id.to_string(),
                    metadata: [("username".into(), username.to_string())].into(),
                    occurred_at: Utc::now(),
                })
                .await?;
            return Err(DomainError::InvalidCredentials);
        }

        let token = self.tokens.issue(user.id, &user.username).await?;
        self.audit
            .append(AuditEventDraft {
                event_type: AuditEventType::LoginSuccess,
                actor_user_id: Some(user.id),
                transfer_id: None,
                request_id: request_id.to_string(),
                trace_id: trace_id.to_string(),
                metadata: std::collections::BTreeMap::new(),
                occurred_at: Utc::now(),
            })
            .await?;

        Ok((token, user.id, user.username))
    }

    /// Returns current user profile.
    pub async fn me(&self, user_id: Uuid) -> Result<(Uuid, String), DomainError> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(DomainError::UserNotFound)?;
        Ok((user.id, user.username))
    }
}
