use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, CreatedResponse, NoContent};

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyCreatedResponse {
    pub id: Uuid,
    pub name: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create(
    Path(_org_id): Path<Uuid>,
    Json(_req): Json<CreateApiKeyRequest>,
) -> CreatedResponse<ApiKeyCreatedResponse> {
    CreatedResponse::new(ApiKeyCreatedResponse {
        id: Uuid::new_v4(),
        name: String::new(),
        key: String::new(),
    })
}

pub async fn list(Path(_org_id): Path<Uuid>) -> ApiResponse<Vec<ApiKeyResponse>> {
    ApiResponse::success(vec![])
}

pub async fn delete(
    Path((_org_id, _key_id)): Path<(Uuid, Uuid)>,
) -> NoContent {
    NoContent
}
