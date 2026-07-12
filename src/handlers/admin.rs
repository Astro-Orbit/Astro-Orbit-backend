use serde::Serialize;
use uuid::Uuid;

use crate::response::ApiResponse;

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub stellar_public: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminOrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub member_count: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminStatsResponse {
    pub total_users: u64,
    pub total_orgs: u64,
    pub total_projects: u64,
    pub total_deployments: u64,
    pub total_contracts: u64,
}

pub async fn users() -> ApiResponse<Vec<AdminUserResponse>> {
    ApiResponse::success(vec![])
}

pub async fn orgs() -> ApiResponse<Vec<AdminOrgResponse>> {
    ApiResponse::success(vec![])
}

pub async fn stats() -> ApiResponse<AdminStatsResponse> {
    ApiResponse::success(AdminStatsResponse {
        total_users: 0,
        total_orgs: 0,
        total_projects: 0,
        total_deployments: 0,
        total_contracts: 0,
    })
}
