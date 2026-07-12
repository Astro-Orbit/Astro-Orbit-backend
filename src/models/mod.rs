//! Domain entity models.
//!
//! Each model is a pure Rust struct representing a database entity.
//! Models contain no business logic and no database access — they
//! are plain data containers with derive macros for SQLx and Serde.
//!
//! Ownership: Data Team
//! Dependencies: none (pure data structures)
//! Public API: All model structs

pub mod contract;
pub mod deployment;
pub mod org;
pub mod project;
pub mod user;
pub mod wallet;
