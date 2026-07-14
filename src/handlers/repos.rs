use std::sync::Arc;

use axum::extract::Path;
use axum::{Extension, Json};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::context::AuthContext;
use crate::dto::repository::*;
use crate::errors::AppError;
use crate::response::{ApiResponse, CreatedResponse, NoContent};
use crate::services::repository_service::RepositoryService;
use crate::state::AppState;

fn build_repository_service(state: &AppState) -> Result<impl RepositoryService, AppError> {
    let pool = state.db()?;
    Ok(crate::services::repository_service::RepositoryServiceImpl::new(
        pool.clone(),
        Arc::new(crate::repositories::repository_repo::PgRepositoryRepository::new(pool.clone())),
    ))
}

pub async fn create(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateRepositoryRequest>,
) -> Result<CreatedResponse<RepositoryResponse>, AppError> {
    let state = state.read().await;
    let service = build_repository_service(&state)?;
    let repo = service
        .link_repository(
            project_id,
            &req.name,
            &req.url,
            &req.provider,
            req.branch.as_deref().unwrap_or("main"),
            req.webhook_secret.as_deref(),
        )
        .await?;

    Ok(CreatedResponse::new(RepositoryResponse {
        id: repo.id,
        project_id: repo.project_id,
        name: repo.name,
        url: repo.url,
        provider: repo.provider,
        branch: repo.branch,
        is_active: repo.is_active,
        last_synced_at: repo.last_synced_at,
        created_at: repo.created_at,
    }))
}

pub async fn list(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(project_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<RepositoryResponse>>, AppError> {
    let state = state.read().await;
    let service = build_repository_service(&state)?;
    let repos = service.list_repositories(project_id).await?;

    Ok(ApiResponse::success(
        repos
            .into_iter()
            .map(|r| RepositoryResponse {
                id: r.id,
                project_id: r.project_id,
                name: r.name,
                url: r.url,
                provider: r.provider,
                branch: r.branch,
                is_active: r.is_active,
                last_synced_at: r.last_synced_at,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub async fn get_by_id(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<RepositoryResponse>, AppError> {
    let state = state.read().await;
    let service = build_repository_service(&state)?;
    let repo = service.get_repository(id).await?;

    Ok(ApiResponse::success(RepositoryResponse {
        id: repo.id,
        project_id: repo.project_id,
        name: repo.name,
        url: repo.url,
        provider: repo.provider,
        branch: repo.branch,
        is_active: repo.is_active,
        last_synced_at: repo.last_synced_at,
        created_at: repo.created_at,
    }))
}

pub async fn update(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRepositoryRequest>,
) -> Result<ApiResponse<RepositoryResponse>, AppError> {
    let state = state.read().await;
    let service = build_repository_service(&state)?;
    let repo = service.update_repository(id, req.name.as_deref(), req.branch.as_deref(), req.is_active).await?;

    Ok(ApiResponse::success(RepositoryResponse {
        id: repo.id,
        project_id: repo.project_id,
        name: repo.name,
        url: repo.url,
        provider: repo.provider,
        branch: repo.branch,
        is_active: repo.is_active,
        last_synced_at: repo.last_synced_at,
        created_at: repo.created_at,
    }))
}

pub async fn delete(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<NoContent, AppError> {
    let state = state.read().await;
    let service = build_repository_service(&state)?;
    service.unlink_repository(id).await?;
    Ok(NoContent)
}

pub async fn sync(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<SyncRepositoryResponse>, AppError> {
    let state = state.read().await;
    let service = build_repository_service(&state)?;
    let repo = service.sync_repository(id).await?;

    Ok(ApiResponse::success(SyncRepositoryResponse {
        id: repo.id,
        status: "synced".to_string(),
        last_synced_at: repo.last_synced_at.unwrap_or_else(chrono::Utc::now),
    }))
}
