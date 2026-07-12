use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

use astro_orbit_backend::config::Config;
use astro_orbit_backend::state::AppState;
use astro_orbit_backend::telemetry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(Config::from_env()?);

    telemetry::init(&config)?;

    let state = startup(&config).await;
    let state = Arc::new(RwLock::new(state));

    let app = astro_orbit_backend::router::build_router(&config, state.clone());

    let addr = SocketAddr::new(config.app.host, config.app.port);
    info!(
        app.name = %config.app.name,
        app.env = %config.app.env,
        addr = %addr,
        "server started"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(config.app.shutdown_timeout))
        .await?;

    Ok(())
}

/// Deterministic startup sequence.
///
/// 1. Load configuration (already done)
/// 2. Initialize telemetry (already done)
/// 3. Connect to database and run migrations
/// 4. Connect to Redis
/// 5. Build shared state
async fn startup(config: &Arc<Config>) -> AppState {
    #[cfg(feature = "db-postgres")]
    let db = match astro_orbit_backend::database::init_pool(&config.database).await {
        Ok(pool) => {
            if let Err(e) = astro_orbit_backend::database::run_migrations(&pool).await {
                tracing::error!(error = %e, "database migration failed");
            }
            Some(pool)
        }
        Err(e) => {
            tracing::error!(error = %e, "database init failed, running without database");
            None
        }
    };

    #[cfg(not(feature = "db-postgres"))]
    let db: Option<std::sync::Arc<sqlx::PgPool>> = None;

    #[cfg(feature = "cache-redis")]
    let cache = match astro_orbit_backend::cache::client::CacheClient::connect(&config.redis).await {
        Ok(client) => Some(Arc::new(client)),
        Err(e) => {
            tracing::error!(error = %e, "cache init failed, running without cache");
            None
        }
    };

    #[cfg(not(feature = "cache-redis"))]
    let cache: Option<Arc<astro_orbit_backend::cache::CacheClient>> = None;

    AppState { config: Arc::clone(config), db, cache }
}

/// Waits for SIGINT or SIGTERM, then initiates graceful shutdown.
async fn shutdown_signal(timeout: std::time::Duration) {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!(timeout_secs = timeout.as_secs(), "shutdown signal received, starting graceful shutdown");

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
