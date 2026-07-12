use axum::extract::Path;
use serde::Serialize;
use uuid::Uuid;

use crate::response::ApiResponse;

#[derive(Debug, Serialize)]
pub struct AnalyticsOverview {
    pub total_transactions: u64,
    pub total_contracts: u64,
    pub total_deployments: u64,
    pub active_users_24h: u64,
    pub gas_used_24h: String,
}

pub async fn overview(Path(_org_id): Path<Uuid>) -> ApiResponse<AnalyticsOverview> {
    ApiResponse::success(AnalyticsOverview {
        total_transactions: 0,
        total_contracts: 0,
        total_deployments: 0,
        active_users_24h: 0,
        gas_used_24h: "0".to_string(),
    })
}

pub async fn contract_calls(Path(_project_id): Path<Uuid>) -> ApiResponse<Vec<serde_json::Value>> {
    ApiResponse::success(vec![])
}

pub async fn gas_usage(Path(_project_id): Path<Uuid>) -> ApiResponse<Vec<serde_json::Value>> {
    ApiResponse::success(vec![])
}

pub async fn active_users(Path(_project_id): Path<Uuid>) -> ApiResponse<Vec<serde_json::Value>> {
    ApiResponse::success(vec![])
}
