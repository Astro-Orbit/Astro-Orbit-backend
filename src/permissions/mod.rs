//! Role-based access control (RBAC) engine.
//!
//! Defines the permission model, role hierarchy, and policy evaluation.
//! Permissions are checked at the middleware layer before requests
//! reach handlers.
//!
//! Ownership: Auth Team
//! Dependencies: models
//! Public API: Authorizer, Permission, Role, check, require

pub mod policy;
pub mod role;
