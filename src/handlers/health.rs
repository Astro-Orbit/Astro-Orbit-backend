use axum::Json;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: &'static str,
    pub uptime: u64,
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub status: String,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

static START_TIME: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

/// GET /v1/health
pub async fn health_check() -> Json<HealthResponse> {
    let uptime = START_TIME.elapsed().as_secs();

    Json(HealthResponse { status: "pass".to_string(), version: env!("CARGO_PKG_VERSION"), uptime })
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: &'static str,
}

/// GET /v1/version
pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse { version: env!("CARGO_PKG_VERSION") })
}
