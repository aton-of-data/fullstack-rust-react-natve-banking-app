use sha2::{Digest, Sha256};

/// Validates idempotency key format (UUID or safe opaque identifier).
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

/// Computes request fingerprint for idempotency conflict detection.
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

/// Hashes idempotency key for safe logging.
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
