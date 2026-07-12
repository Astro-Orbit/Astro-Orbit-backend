use std::sync::Arc;

use crate::config::Config;

#[cfg(feature = "db-postgres")]
use sqlx::PgPool;

#[cfg(feature = "cache-redis")]
use crate::cache::client::CacheClient;

/// Centralized application state shared across all handlers.
///
/// Every dependency is behind an `Option` + `Arc` so that the application
/// can start even when optional infrastructure (database, cache, metrics)
/// is unavailable.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,

    #[cfg(feature = "db-postgres")]
    pub db: Option<Arc<PgPool>>,

    #[cfg(feature = "cache-redis")]
    pub cache: Option<Arc<CacheClient>>,
}

impl AppState {
    /// Returns a reference to the database pool, if configured.
    #[cfg(feature = "db-postgres")]
    pub fn db(&self) -> Result<Arc<PgPool>, crate::errors::AppError> {
        self.db.clone().ok_or_else(|| crate::errors::AppError::unavailable("database not configured"))
    }

    /// Returns a reference to the cache client, if configured.
    #[cfg(feature = "cache-redis")]
    pub fn cache(&self) -> Result<Arc<CacheClient>, crate::errors::AppError> {
        self.cache.clone().ok_or_else(|| crate::errors::AppError::unavailable("cache not configured"))
    }
}
