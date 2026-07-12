//! Data access layer — repository pattern.
//!
//! Repositories abstract database access behind traits. Each domain
//! has its own repository trait and implementation. This enables
//! unit-testing services with mock repositories.
//!
//! Ownership: Data Team
//! Dependencies: models, database
//! Public API: All repository traits and implementations

pub mod contract_repo;
pub mod deployment_repo;
pub mod org_repo;
pub mod project_repo;
pub mod user_repo;
pub mod wallet_repo;
