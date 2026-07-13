use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use stellar_strkey::ed25519::PublicKey;

use crate::errors::AppError;

/// Verifies a Stellar wallet's Ed25519 signature against a challenge.
///
/// Accepts a Stellar public key (G...) and hex-encoded signature.
/// Uses `stellar-strkey` to decode the G... key into a raw Ed25519
/// public key, then verifies with `ed25519-dalek`.
pub fn verify_stellar_signature(public_key: &str, challenge: &str, signature_hex: &str) -> Result<(), AppError> {
    let pk = decode_public_key(public_key)?;

    let sig_bytes = hex::decode(signature_hex).map_err(|_| AppError::bad_request("invalid signature hex encoding"))?;

    let signature = Signature::from_slice(&sig_bytes).map_err(|_| AppError::bad_request("invalid signature format"))?;

    pk.verify(challenge.as_bytes(), &signature).map_err(|_| AppError::Unauthenticated)?;

    Ok(())
}

/// Decodes a Stellar public key (G...) into an Ed25519 verifying key.
fn decode_public_key(stellar_key: &str) -> Result<VerifyingKey, AppError> {
    let decoded: PublicKey =
        stellar_key.parse().map_err(|_| AppError::bad_request("invalid Stellar public key format"))?;

    VerifyingKey::from_bytes(&decoded.0).map_err(|_| AppError::internal("failed to construct Ed25519 verifying key"))
}

#[must_use]
pub fn validate_public_key_format(key: &str) -> bool {
    if !key.starts_with('G') || key.len() != 56 {
        return false;
    }
    key.parse::<PublicKey>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_public_key_format_valid() {
        let valid = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
        assert!(validate_public_key_format(valid));
    }

    #[test]
    fn test_validate_public_key_format_invalid_prefix() {
        assert!(!validate_public_key_format("S..."));
    }

    #[test]
    fn test_validate_public_key_format_wrong_length() {
        assert!(!validate_public_key_format("G12345"));
    }
}
