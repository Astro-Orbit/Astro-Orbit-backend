use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::config::DatabaseConfig;
use crate::errors::AppError;

/// Initializes the database connection pool.
///
/// # Errors
///
/// Returns `AppError::Unavailable` if the pool cannot connect to the database
/// within the configured timeout.
pub async fn init_pool(config: &DatabaseConfig) -> Result<Arc<PgPool>, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .max_lifetime(config.max_lifetime)
        .idle_timeout(config.idle_timeout)
        .acquire_timeout(config.connect_timeout)
        .test_before_acquire(true)
        .connect(&config.url)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, config.url = %config.url, "failed to connect to database");
            AppError::unavailable(format!("database connection failed: {e}"))
        })?;

    tracing::info!(
        max_connections = config.max_connections,
        min_connections = config.min_connections,
        "database connection pool established"
    );

    Ok(Arc::new(pool))
}

/// Runs all pending database migrations.
///
/// # Errors
///
/// Returns `AppError::Internal` if migrations fail.
pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations").run(pool).await.map_err(|e| {
        tracing::error!(error = %e, "failed to run database migrations");
        AppError::internal(format!("database migration failed: {e}"))
    })?;

    tracing::info!("database migrations applied successfully");
    Ok(())
}

/// Checks database connectivity by executing `SELECT 1`.
///
/// Returns `Ok(())` on success, or an `AppError::Unavailable` on failure
/// with the measured latency.
pub async fn check_health(pool: &PgPool) -> Result<Duration, AppError> {
    let start = std::time::Instant::now();
    sqlx::query("SELECT 1").execute(pool).await.map_err(|e| {
        tracing::warn!(error = %e, "database health check failed");
        AppError::unavailable(format!("database health check failed: {e}"))
    })?;
    Ok(start.elapsed())
}

/// Returns pool statistics for monitoring.
#[must_use]
pub fn pool_status(pool: &PgPool) -> PoolStatus {
    let size = pool.size();
    let num_idle = pool.num_idle() as u32;
    PoolStatus { size, num_idle, num_active: size - num_idle, max_size: pool.options().get_max_connections() }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStatus {
    pub size: u32,
    pub num_idle: u32,
    pub num_active: u32,
    pub max_size: u32,
}

/// Runs a database transaction with automatic rollback on error.
///
/// # Errors
///
/// Propagates errors from the database layer.
pub async fn with_transaction<T, F, E>(pool: &PgPool, f: F) -> Result<T, E>
where
    F: for<'a> FnOnce(&mut sqlx::Transaction<'a, sqlx::Postgres>) -> Result<T, E>,
    E: From<sqlx::Error> + std::fmt::Display,
{
    let mut tx = pool.begin().await?;
    match f(&mut tx) {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tracing::warn!(error = %e, "rolling back database transaction");
            tx.rollback().await?;
            Err(e)
        }
    }
}

/// Runs a database transaction asynchronously with automatic rollback on error.
///
/// # Errors
///
/// Propagates errors from the database layer.
pub async fn with_transaction_async<T, F, Fut, E>(pool: &PgPool, f: F) -> Result<T, E>
where
    F: for<'a> FnOnce(&mut sqlx::Transaction<'a, sqlx::Postgres>) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: From<sqlx::Error>,
{
    let mut tx = pool.begin().await?;
    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}
