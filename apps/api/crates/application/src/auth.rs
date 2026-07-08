//! Authentication use cases.
//!
//! [`AuthService`] verifies credentials and issues access tokens through ports.
//! It does not implement Argon2, JWT crypto, or HTTP cookie/header handling —
//! those belong to infrastructure and `adapters-http`.
//!
//! # Security invariants
//!
//! - Password verification goes through [`PasswordHasher`] only; plaintext is
//!   never hashed or compared inline in this module.
//! - Both unknown usernames and bad passwords yield the same
//!   [`DomainError::InvalidCredentials`] so callers cannot user-enumerate via
//!   distinct error variants from this service.
//! - Audit events record success/failure with request/trace correlation and
//!   (on failure) username metadata only — **never the password**.

use std::sync::Arc;

use chrono::Utc;
use ficus_domain::audit::{AuditEventDraft, AuditEventType};
use ficus_domain::errors::DomainError;
use uuid::Uuid;

use crate::ports::{AuditRepository, PasswordHasher, TokenService, UserRepository};

/// Login and current-user profile use cases.
///
/// Depends on user lookup, password verification, token issuance, and audit
/// append ports. Constructed once at the composition root.
pub struct AuthService {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    audit: Arc<dyn AuditRepository>,
}

impl AuthService {
    /// Creates an auth service over the given ports.
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

    /// Authenticates `username` / `password` and returns `(access_token, user_id, username)`.
    ///
    /// # Flow
    ///
    /// 1. Look up the user by username; missing user →
    ///    [`DomainError::InvalidCredentials`] (no audit distinguishability of
    ///    “unknown user” vs “bad password” in the returned error).
    /// 2. Verify the password via [`PasswordHasher::verify`].
    /// 3. On failure: append `LoginFailure` audit (username in metadata only),
    ///    then return [`DomainError::InvalidCredentials`].
    /// 4. On success: issue a token, append `LoginSuccess` audit with
    ///    `actor_user_id`, return token and identity.
    ///
    /// # Security
    ///
    /// Do not log `password`. Audit metadata must remain free of secrets.
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

    /// Returns `(user_id, username)` for the authenticated subject.
    ///
    /// Fails with [`DomainError::UserNotFound`] if the id no longer resolves
    /// (e.g. deleted user with a still-valid token until expiry).
    pub async fn me(&self, user_id: Uuid) -> Result<(Uuid, String), DomainError> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(DomainError::UserNotFound)?;
        Ok((user.id, user.username))
    }
}
