//! JWT token issuance and verification.
//!
//! Uses `jsonwebtoken` with HS256 or `EdDSA` signing.
//! Tokens contain `user_id`, `session_id`, and active organization context.
