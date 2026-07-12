use axum::extract::Path;
use serde::Serialize;
use uuid::Uuid;

use crate::response::ApiResponse;

#[derive(Debug, Serialize)]
pub struct ExplorerContract {
    pub id: Uuid,
    pub contract_id: String,
    pub wasm_hash: Option<String>,
    pub code: Option<String>,
}

pub async fn contract_details(Path(_id): Path<Uuid>) -> ApiResponse<ExplorerContract> {
    ApiResponse::success(ExplorerContract {
        id: Uuid::new_v4(),
        contract_id: String::new(),
        wasm_hash: None,
        code: None,
    })
}

pub async fn transactions() -> ApiResponse<Vec<serde_json::Value>> {
    ApiResponse::success(vec![])
}

pub async fn transaction_details(Path(_id): Path<Uuid>) -> ApiResponse<serde_json::Value> {
    ApiResponse::success(serde_json::json!({}))
}

pub async fn events() -> ApiResponse<Vec<serde_json::Value>> {
    ApiResponse::success(vec![])
}

pub async fn event_details(Path(_id): Path<Uuid>) -> ApiResponse<serde_json::Value> {
    ApiResponse::success(serde_json::json!({}))
}
