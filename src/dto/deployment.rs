use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub environment: Option<String>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub environment: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}
