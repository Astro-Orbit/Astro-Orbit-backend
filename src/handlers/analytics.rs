use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::Extension;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::context::AuthContext;
use crate::errors::AppError;
use crate::response::ApiResponse;
use crate::services::analytics_service::AnalyticsService;
use crate::state::AppState;

fn build_analytics_service(state: &AppState) -> Result<impl AnalyticsService, AppError> {
    let pool = state.db()?;
    Ok(crate::services::analytics_service::AnalyticsServiceImpl::new(
        Arc::new(crate::repositories::org_repo::PgOrgRepository::new(pool.clone())),
        Arc::new(crate::repositories::project_repo::PgProjectRepository::new(pool.clone())),
        Arc::new(crate::repositories::contract_repo::PgContractRepository::new(pool.clone())),
        Arc::new(crate::repositories::deployment_repo::PgDeploymentRepository::new(pool.clone())),
        Arc::new(crate::repositories::activity_repo::PgActivityRepository::new(pool.clone())),
    ))
}

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

// --- Dashboard endpoints ---

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_projects: i64,
    pub total_contracts: i64,
    pub total_deployments: i64,
    pub active_deployments: i64,
    pub total_members: i64,
    pub activities_24h: i64,
}

pub async fn dashboard_stats(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(org_id): Path<Uuid>,
) -> Result<ApiResponse<DashboardStats>, AppError> {
    let state = state.read().await;
    let service = build_analytics_service(&state)?;
    let stats = service.org_stats(org_id).await?;

    Ok(ApiResponse::success(DashboardStats {
        total_projects: stats.total_projects,
        total_contracts: stats.total_contracts,
        total_deployments: stats.total_deployments,
        active_deployments: stats.active_deployments,
        total_members: stats.total_members,
        activities_24h: stats.activities_24h,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ActivityFeedParams {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ActivityFeedItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub actor_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn dashboard_activity(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Query(params): Query<ActivityFeedParams>,
) -> Result<ApiResponse<Vec<ActivityFeedItem>>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let state = state.read().await;
    let service = build_analytics_service(&state)?;
    let activities = service.user_activity_feed(auth.user.id, limit).await?;

    Ok(ApiResponse::success(
        activities
            .into_iter()
            .map(|a| ActivityFeedItem {
                id: a.id,
                organization_id: a.organization_id,
                actor_id: a.actor_id,
                action: a.action,
                resource_type: a.resource_type,
                resource_id: a.resource_id,
                metadata: a.metadata,
                created_at: a.created_at,
            })
            .collect(),
    ))
}
