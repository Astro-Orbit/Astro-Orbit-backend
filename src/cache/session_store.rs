//! Session store — re-exported from auth module.
//!
//! The primary session management lives in `crate::auth::session`.
//! This module re-exports for backward compatibility with existing
//! imports that reference the cache module directly.

pub use crate::auth::session::{SessionData, SessionStore};
