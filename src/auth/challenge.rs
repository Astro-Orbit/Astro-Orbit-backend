use uuid::Uuid;

use crate::errors::AppError;
use crate::utils::crypto::random_hex;

pub const CHALLENGE_BYTES: usize = 32;

/// Generates a cryptographically random challenge string.
#[must_use]
pub fn generate_challenge() -> String {
    random_hex::<CHALLENGE_BYTES>()
}

/// Validates that a challenge is still valid (not expired, not already used).
pub fn validate_challenge(
    expires_at: &chrono::DateTime<chrono::Utc>,
    used_at: &Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Uuid, AppError> {
    if used_at.is_some() {
        return Err(AppError::conflict("challenge already used"));
    }

    if chrono::Utc::now() > *expires_at {
        return Err(AppError::bad_request("challenge expired"));
    }

    Ok(Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_generate_challenge_length() {
        let challenge = generate_challenge();
        assert_eq!(challenge.len(), CHALLENGE_BYTES * 2);
    }

    #[test]
    fn test_validate_challenge_valid() {
        let expires = Utc::now() + Duration::minutes(5);
        let result = validate_challenge(&expires, &None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_challenge_expired() {
        let expires = Utc::now() - Duration::seconds(1);
        let result = validate_challenge(&expires, &None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_challenge_used() {
        let expires = Utc::now() + Duration::minutes(5);
        let used = Some(Utc::now());
        let result = validate_challenge(&expires, &used);
        assert!(result.is_err());
    }
}
