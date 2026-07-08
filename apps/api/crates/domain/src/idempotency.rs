//! Idempotency key validation and request fingerprinting.
//!
//! This module belongs to the domain layer. It owns pure string/hash helpers
//! used by the application service and HTTP boundary. It does **not** store
//! responses, acquire advisory locks, or decide conflict vs replay — those are
//! application and persistence concerns.
//!
//! # Idempotency model
//!
//! An idempotency key is a client-supplied retry token scoped to the sending
//! user. Combined with a payload fingerprint, it distinguishes:
//!
//! - **Same key + same fingerprint** → safe retry / replay of the original
//!   result (no second charge).
//! - **Same key + different fingerprint** → conflict (must not charge a
//!   different payload under a reused key).

use sha2::{Digest, Sha256};

/// Validates an idempotency key format (UUID or safe opaque identifier).
///
/// Accepted forms:
///
/// - Any string parseable as a UUID
/// - Opaque keys of length 8..=128 consisting of ASCII alphanumeric characters,
///   `-`, and `_` only
///
/// Empty keys and keys longer than 128 characters are rejected.
///
/// # Examples
///
/// ```
/// use ficus_domain::idempotency::validate_idempotency_key;
///
/// assert!(validate_idempotency_key(
///     "550e8400-e29b-41d4-a716-446655440000"
/// ));
/// assert!(validate_idempotency_key("retry-key-01"));
/// assert!(!validate_idempotency_key(""));
/// assert!(!validate_idempotency_key("short"));
/// ```
pub fn validate_idempotency_key(key: &str) -> bool {
    let trimmed = key.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return false;
    }
    // UUID format
    if uuid::Uuid::parse_str(trimmed).is_ok() {
        return true;
    }
    // Safe opaque: alphanumeric, dash, underscore
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && trimmed.len() >= 8
}

/// Computes a SHA-256 hex fingerprint of the logical transfer request.
///
/// The fingerprint binds an idempotency key's protected payload to:
///
/// - sender user id
/// - recipient username
/// - amount (minor units string as received)
/// - currency code
/// - description (empty string when absent)
///
/// Reusing the same key with the same fingerprint is treated as a retry.
/// Reusing the same key with a different fingerprint is a conflict because it
/// could otherwise charge a user for a request they did not intend to retry.
///
/// # Security
///
/// The fingerprint is a hash of business fields, not a secret. Do not log raw
/// idempotency keys; use [`hash_idempotency_key`] for log correlation.
///
/// # Examples
///
/// ```
/// use ficus_domain::idempotency::request_fingerprint;
///
/// let a = request_fingerprint("u1", "bob", "100", "USD", "lunch");
/// let b = request_fingerprint("u1", "bob", "200", "USD", "lunch");
/// assert_ne!(a, b);
/// assert_eq!(a.len(), 64); // SHA-256 hex
/// ```
pub fn request_fingerprint(
    sender_user_id: &str,
    recipient_username: &str,
    amount_minor: &str,
    currency: &str,
    description: &str,
) -> String {
    let payload =
        format!("{sender_user_id}|{recipient_username}|{amount_minor}|{currency}|{description}");
    let hash = Sha256::digest(payload.as_bytes());
    hex::encode(hash)
}

/// Hashes an idempotency key for safe logging and audit correlation.
///
/// Returns the first 8 bytes of SHA-256 encoded as 16 hex characters. Never
/// logs the raw key in production paths.
///
/// # Examples
///
/// ```
/// use ficus_domain::idempotency::hash_idempotency_key;
///
/// let hashed = hash_idempotency_key("550e8400-e29b-41d4-a716-446655440000");
/// assert_eq!(hashed.len(), 16);
/// ```
pub fn hash_idempotency_key(key: &str) -> String {
    let hash = Sha256::digest(key.as_bytes());
    hex::encode(&hash[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_uuid_key() {
        assert!(validate_idempotency_key(
            "550e8400-e29b-41d4-a716-446655440000"
        ));
    }

    #[test]
    fn rejects_empty_key() {
        assert!(!validate_idempotency_key(""));
    }

    #[test]
    fn fingerprint_changes_with_payload() {
        let a = request_fingerprint("u1", "bob", "100", "USD", "lunch");
        let b = request_fingerprint("u1", "bob", "200", "USD", "lunch");
        assert_ne!(a, b);
    }
}
