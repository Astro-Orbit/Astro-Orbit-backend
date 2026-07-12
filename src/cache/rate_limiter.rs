//! Token bucket rate limiter backed by Redis.
//!
//! Uses Lua scripts for atomic increment + expiry operations.
//! Supports per-user, per-IP, and per-endpoint rate limits.
