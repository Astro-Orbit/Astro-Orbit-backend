//! Deployment pipeline orchestration.
//!
//! Manages the full deployment lifecycle as a state machine:
//!   pending -> building -> scanning -> deploying -> deployed
//!                                         -> failed
//!              -> cancelled
//!
//! Each deployment transitions through stages with associated
//! validations, security scans, and Soroban RPC calls.
//!
//! Ownership: Deployment Team
//! Dependencies: stellar, services, events, cache
//! Public API: `DeploymentService`, `DeploymentState`, `create_deployment`, `rollback_deployment`

pub mod pipeline;
pub mod state;
pub mod validator;
