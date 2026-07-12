//! Redis-backed caching layer.
//!
//! Provides a unified cache abstraction for:
//!   - Session storage
//!   - Rate limit counters
//!   - Computed/computed-on-demand values
//!   - Auth challenge nonces
//!
//! Ownership: Infrastructure Team
//! Dependencies: config (`RedisConfig`)
//! Public API: `CacheClient`, get, set, delete, exists, ttl

pub mod client;
pub mod rate_limiter;
pub mod session_store;
