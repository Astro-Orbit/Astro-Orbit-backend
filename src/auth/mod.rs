//! Wallet-based authentication and session management.
//!
//! This module handles the Stellar wallet authentication flow:
//!   1. Challenge generation (signed message request)
//!   2. Signature verification (Ed25519 via ed25519-dalek)
//!   3. JWT access token issuance and validation
//!   4. Refresh token rotation
//!   5. Session management and revocation
//!
//! Ownership: Auth Team
//! Dependencies: config, cache, repositories
//! Public API: issue_jwt, verify_jwt, generate_challenge, verify_signature

pub const ACCESS_TOKEN_TTL_BUFFER_SECONDS: u64 = 30;
pub const REFRESH_TOKEN_BYTES: usize = 32;
pub const CHALLENGE_BYTES: usize = 32;

pub mod jwt;
pub mod session;
pub mod wallet;
