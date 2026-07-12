/// Validates that a string is a valid Stellar public key (G...).
pub fn validate_public_key(key: &str) -> bool {
    key.starts_with('G') && key.len() == 56
}

/// Validates that a string is a valid Stellar contract ID (C...).
pub fn validate_contract_id(id: &str) -> bool {
    id.starts_with('C') && id.len() == 56
}
