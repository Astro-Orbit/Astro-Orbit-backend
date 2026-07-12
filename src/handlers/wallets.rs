use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, CreatedResponse, NoContent};

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub public_key: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub id: Uuid,
    pub public_key: String,
    pub name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWalletRequest {
    pub name: Option<String>,
}

pub async fn create(Json(_req): Json<CreateWalletRequest>) -> CreatedResponse<WalletResponse> {
    CreatedResponse::new(WalletResponse {
        id: Uuid::new_v4(),
        public_key: String::new(),
        name: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn list() -> ApiResponse<Vec<WalletResponse>> {
    ApiResponse::success(vec![])
}

pub async fn get_by_id(Path(_id): Path<Uuid>) -> ApiResponse<WalletResponse> {
    ApiResponse::success(WalletResponse {
        id: Uuid::new_v4(),
        public_key: String::new(),
        name: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn update(Path(_id): Path<Uuid>, Json(_req): Json<UpdateWalletRequest>) -> ApiResponse<WalletResponse> {
    ApiResponse::success(WalletResponse {
        id: Uuid::new_v4(),
        public_key: String::new(),
        name: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn delete(Path(_id): Path<Uuid>) -> NoContent {
    NoContent
}
