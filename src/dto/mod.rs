//! Data Transfer Objects — request validation models.
//!
//! Every incoming request has a corresponding DTO struct with
//! serde deserialization and validator derive annotations.
//! These live at the handler boundary and are never passed
//! directly to repositories.
//!
//! Ownership: API Team
//! Dependencies: models, validation
//! Public API: All request/response types within each submodule

pub mod auth;
pub mod contract;
pub mod deployment;
pub mod notification;
pub mod org;
pub mod project;
pub mod user;
pub mod wallet;
