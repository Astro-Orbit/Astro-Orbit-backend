use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::DatabaseConfig;

/// Initializes the database connection pool.
pub async fn init_pool(config: &DatabaseConfig) -> Arc<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .max_lifetime(config.max_lifetime)
        .idle_timeout(config.idle_timeout)
        .acquire_timeout(config.connect_timeout)
        .connect(&config.url)
        .await
        .expect("failed to connect to database");

    tracing::info!(max_connections = config.max_connections, "database connection pool established");

    Arc::new(pool)
}

/// Runs all pending database migrations.
pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations").run(pool).await.expect("failed to run database migrations");

    tracing::info!("database migrations applied successfully");
}
