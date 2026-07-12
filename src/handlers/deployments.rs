use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, CreatedResponse, NoContent};

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub environment: Option<String>,
    pub contract_ids: Vec<Uuid>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
    pub commit_message: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub environment: String,
    pub status: String,
    pub version: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create(
    Path(_project_id): Path<Uuid>,
    Json(_req): Json<CreateDeploymentRequest>,
) -> CreatedResponse<DeploymentResponse> {
    CreatedResponse::new(DeploymentResponse {
        id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        environment: "development".to_string(),
        status: "pending".to_string(),
        version: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn list(Path(_project_id): Path<Uuid>) -> ApiResponse<Vec<DeploymentResponse>> {
    ApiResponse::success(vec![])
}

pub async fn get_by_id(Path(_id): Path<Uuid>) -> ApiResponse<DeploymentResponse> {
    ApiResponse::success(DeploymentResponse {
        id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        environment: "development".to_string(),
        status: "pending".to_string(),
        version: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn rollback(Path(_id): Path<Uuid>) -> ApiResponse<DeploymentResponse> {
    ApiResponse::success(DeploymentResponse {
        id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        environment: "development".to_string(),
        status: "rolled_back".to_string(),
        version: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn cancel(Path(_id): Path<Uuid>) -> NoContent {
    NoContent
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}

pub async fn logs(Path(_id): Path<Uuid>) -> ApiResponse<Vec<LogEntry>> {
    ApiResponse::success(vec![])
}
