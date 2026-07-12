//! Integration tests for repository and service layers.
//!
//! These tests run against a real PostgreSQL database (via Docker
//! container or dedicated test database). Each test runs inside
//! its own database transaction that rolls back on completion.
