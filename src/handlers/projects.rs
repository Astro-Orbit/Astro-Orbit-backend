use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, CreatedResponse, NoContent};

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub network: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub network: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub async fn create(
    Path(_org_id): Path<Uuid>,
    Json(_req): Json<CreateProjectRequest>,
) -> CreatedResponse<ProjectResponse> {
    CreatedResponse::new(ProjectResponse {
        id: Uuid::new_v4(),
        organization_id: Uuid::new_v4(),
        name: String::new(),
        slug: String::new(),
        description: None,
        network: "testnet".to_string(),
        created_at: chrono::Utc::now(),
    })
}

pub async fn list(Path(_org_id): Path<Uuid>) -> ApiResponse<Vec<ProjectResponse>> {
    ApiResponse::success(vec![])
}

pub async fn get_by_id(Path(_id): Path<Uuid>) -> ApiResponse<ProjectResponse> {
    ApiResponse::success(ProjectResponse {
        id: Uuid::new_v4(),
        organization_id: Uuid::new_v4(),
        name: String::new(),
        slug: String::new(),
        description: None,
        network: "testnet".to_string(),
        created_at: chrono::Utc::now(),
    })
}

pub async fn update(
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateProjectRequest>,
) -> ApiResponse<ProjectResponse> {
    ApiResponse::success(ProjectResponse {
        id: Uuid::new_v4(),
        organization_id: Uuid::new_v4(),
        name: String::new(),
        slug: String::new(),
        description: None,
        network: "testnet".to_string(),
        created_at: chrono::Utc::now(),
    })
}

pub async fn delete(Path(_id): Path<Uuid>) -> NoContent {
    NoContent
}
