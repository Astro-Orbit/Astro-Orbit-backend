use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, CreatedResponse};

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub name: String,
    pub contract_id: String,
    pub wasm_hash: Option<String>,
    pub source_language: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContractResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub contract_id: String,
    pub wasm_hash: Option<String>,
    pub verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContractRequest {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVersionRequest {
    pub wasm_hash: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ContractVersionResponse {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub version: String,
    pub wasm_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create(
    Path(_project_id): Path<Uuid>,
    Json(_req): Json<CreateContractRequest>,
) -> CreatedResponse<ContractResponse> {
    CreatedResponse::new(ContractResponse {
        id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        name: String::new(),
        contract_id: String::new(),
        wasm_hash: None,
        verified: false,
        created_at: chrono::Utc::now(),
    })
}

pub async fn list(Path(_project_id): Path<Uuid>) -> ApiResponse<Vec<ContractResponse>> {
    ApiResponse::success(vec![])
}

pub async fn get_by_id(Path(_id): Path<Uuid>) -> ApiResponse<ContractResponse> {
    ApiResponse::success(ContractResponse {
        id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        name: String::new(),
        contract_id: String::new(),
        wasm_hash: None,
        verified: false,
        created_at: chrono::Utc::now(),
    })
}

pub async fn update(
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateContractRequest>,
) -> ApiResponse<ContractResponse> {
    ApiResponse::success(ContractResponse {
        id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        name: String::new(),
        contract_id: String::new(),
        wasm_hash: None,
        verified: false,
        created_at: chrono::Utc::now(),
    })
}

pub async fn create_version(
    Path(_contract_id): Path<Uuid>,
    Json(_req): Json<CreateVersionRequest>,
) -> CreatedResponse<ContractVersionResponse> {
    CreatedResponse::new(ContractVersionResponse {
        id: Uuid::new_v4(),
        contract_id: Uuid::new_v4(),
        version: String::new(),
        wasm_hash: String::new(),
        created_at: chrono::Utc::now(),
    })
}

pub async fn list_versions(
    Path(_contract_id): Path<Uuid>,
) -> ApiResponse<Vec<ContractVersionResponse>> {
    ApiResponse::success(vec![])
}
