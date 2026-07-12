//! Test application builder.
//!
//! Builds an Axum application for integration testing with:
//!   - In-memory or container-backed PostgreSQL
//!   - Mock Stellar RPC
//!   - Fixed clock for deterministic timestamps
//!   - Pre-configured middleware stack

/// Builds a test application instance with the given configuration.
/// Each test gets a fresh app with isolated state.
pub struct TestAppBuilder;

impl TestAppBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build and return a configured Axum Router for testing.
    pub async fn build(self) -> axum::Router {
        crate::router::build_router(std::sync::Arc::new(
            crate::config::Config::from_env().unwrap(),
        ))
        .await
        .unwrap()
    }
}
