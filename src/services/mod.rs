//! Business logic layer — service orchestration.
//!
//! Services contain all domain business logic. They coordinate
//! between repositories, external clients, and event emission.
//! Services are stateless and receive their dependencies via
//! constructor injection.
//!
//! Ownership: Domain Teams
//! Dependencies: repositories, events, cache, stellar
//! Public API: All public service methods

pub mod api_key_service;
pub mod auth_service;
pub mod contract_service;
pub mod deployment_service;
pub mod notification_service;
pub mod org_service;
pub mod project_service;
pub mod user_service;
pub mod wallet_service;
