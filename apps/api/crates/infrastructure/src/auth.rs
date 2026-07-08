//! Password hashing and JWT access-token adapters for application ports.
//!
//! Implements [`ficus_application::ports::PasswordHasher`] and
//! [`ficus_application::ports::TokenService`] using Argon2id and `jsonwebtoken`.
//! Secrets and TTLs come from configuration; this module does not invent
//! defaults for production secrets.

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher as ArgonHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use ficus_application::ports::{PasswordHasher, TokenService};
use ficus_domain::errors::DomainError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Argon2id password hasher implementing the application [`PasswordHasher`] port.
pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| DomainError::Validation(format!("hash failed: {e}")))
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError> {
        let parsed = PasswordHash::new(hash)
            .map_err(|e| DomainError::Validation(format!("invalid hash: {e}")))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    }
}

/// JWT claims payload (subject, username, issued-at, expiry).
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    exp: i64,
    iat: i64,
}

/// JWT access token service implementing the application [`TokenService`] port.
pub struct JwtTokenService {
    secret: String,
    expiry_secs: u64,
}

impl JwtTokenService {
    /// Creates a service with the given HMAC secret and token lifetime in seconds.
    pub fn new(secret: String, expiry_secs: u64) -> Self {
        Self {
            secret,
            expiry_secs,
        }
    }
}

#[async_trait]
impl TokenService for JwtTokenService {
    async fn issue(&self, user_id: Uuid, username: &str) -> Result<String, DomainError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.expiry_secs as i64)).timestamp(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| DomainError::Validation(format!("token issue failed: {e}")))
    }

    async fn verify(&self, token: &str) -> Result<(Uuid, String), DomainError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| DomainError::Validation("invalid token".into()))?;
        let user_id = Uuid::parse_str(&data.claims.sub)
            .map_err(|_| DomainError::Validation("invalid token subject".into()))?;
        Ok((user_id, data.claims.username))
    }
}
