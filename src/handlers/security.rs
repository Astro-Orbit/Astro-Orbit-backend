use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, CreatedResponse};

#[derive(Debug, Deserialize)]
pub struct CreateScanRequest {
    pub contract_version_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub status: String,
    pub severity: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct FindingResponse {
    pub id: Uuid,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: Option<String>,
}

pub async fn create_scan(
    Path(_contract_id): Path<Uuid>,
    Json(_req): Json<CreateScanRequest>,
) -> CreatedResponse<ScanResponse> {
    CreatedResponse::new(ScanResponse {
        id: Uuid::new_v4(),
        contract_id: Uuid::new_v4(),
        status: "pending".to_string(),
        severity: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn list_scans(Path(_contract_id): Path<Uuid>) -> ApiResponse<Vec<ScanResponse>> {
    ApiResponse::success(vec![])
}

pub async fn get_scan(Path(_id): Path<Uuid>) -> ApiResponse<ScanResponse> {
    ApiResponse::success(ScanResponse {
        id: Uuid::new_v4(),
        contract_id: Uuid::new_v4(),
        status: "pending".to_string(),
        severity: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn findings(Path(_scan_id): Path<Uuid>) -> ApiResponse<Vec<FindingResponse>> {
    ApiResponse::success(vec![])
}
