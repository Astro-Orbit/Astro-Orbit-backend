use std::sync::Arc;
use std::time::Instant;

use axum::Extension;
use axum::Json;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::state::AppState;

static START_TIME: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

/// GET /v1/health — basic liveness check.
#[utoipa::path(
    get,
    path = "/v1/health",
    responses(
        (status = 200, description = "Service is alive"),
    ),
    tag = "Health",
)]
pub async fn health_check() -> Json<HealthResponse> {
    let uptime = START_TIME.elapsed().as_secs();
    Json(HealthResponse { status: "pass".to_string(), version: env!("CARGO_PKG_VERSION"), uptime })
}

/// GET /v1/ready — readiness probe that checks dependencies.
#[utoipa::path(
    get,
    path = "/v1/ready",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready"),
    ),
    tag = "Health",
)]
pub async fn ready_check(Extension(state): Extension<Arc<RwLock<AppState>>>) -> Json<serde_json::Value> {
    let state = state.read().await;
    let mut checks = Vec::new();
    let mut all_healthy = true;

    #[cfg(feature = "db-postgres")]
    if let Some(ref pool) = state.db {
        match crate::database::check_health(pool).await {
            Ok(latency) => {
                checks.push(CheckResult {
                    name: "database",
                    status: "pass",
                    latency_ms: latency.as_millis() as u64,
                    error: None,
                });
            }
            Err(e) => {
                all_healthy = false;
                checks.push(CheckResult {
                    name: "database",
                    status: "fail",
                    latency_ms: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Json(serde_json::json!({
        "status": if all_healthy { "pass" } else { "fail" },
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": START_TIME.elapsed().as_secs(),
        "checks": checks,
    }))
}

/// GET /v1/live — simple liveness probe (always passes if server is running).
#[utoipa::path(
    get,
    path = "/v1/live",
    responses(
        (status = 200, description = "Service is alive"),
    ),
    tag = "Health",
)]
pub async fn live_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "pass",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": START_TIME.elapsed().as_secs(),
    }))
}

/// GET /v1/version — returns the current version.
#[utoipa::path(
    get,
    path = "/v1/version",
    responses(
        (status = 200, description = "Version information"),
    ),
    tag = "Health",
)]
pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION"),
        commit: option_env!("GIT_COMMIT_HASH").unwrap_or("unknown"),
        build_time: option_env!("BUILD_TIMESTAMP").unwrap_or("unknown"),
    })
}

// --- Response types ---

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: &'static str,
    pub uptime: u64,
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: &'static str,
    pub status: &'static str,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: &'static str,
    pub commit: &'static str,
    pub build_time: &'static str,
}
