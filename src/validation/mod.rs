//! Input validation and sanitization utilities.
//!
//! Custom validators for domain-specific constraints that go beyond
//! what the `validator` crate provides. Includes Stellar address
//! validation, slug format checks, and environment name validation.
//!
//! Ownership: Platform Team
//! Dependencies: none
//! Public API: All validation functions

pub mod stellar;
pub mod slug;
pub mod environment;
