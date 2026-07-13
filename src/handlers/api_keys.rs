use std::sync::Arc;

use axum::extract::Path;
use axum::{Extension, Json};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::context::AuthContext;
use crate::dto::api_key::*;
use crate::errors::AppError;
use crate::response::{ApiResponse, CreatedResponse, NoContent};
use crate::services::api_key_service::ApiKeyService;
use crate::state::AppState;

fn build_service(state: &AppState) -> Result<ApiKeyService, AppError> {
    let pool = state.db()?;
    Ok(ApiKeyService::new(
        pool.clone(),
        Arc::new(crate::repositories::api_key_repo::PgApiKeyRepository::new(pool.clone())),
    ))
}

pub async fn create(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<CreatedResponse<CreateApiKeyResponse>, AppError> {
    let org_id = auth.organization.as_ref().map(|o| o.id);

    let state = state.read().await;
    let service = build_service(&state)?;
    let result = service.create(org_id, auth.user.id, &req.name, &req.scopes, req.expires_in_days).await?;

    Ok(CreatedResponse::new(CreateApiKeyResponse {
        id: result.id,
        name: result.name,
        key: result.key,
        prefix: result.prefix,
        scopes: result.scopes,
        expires_at: result.expires_at,
    }))
}

pub async fn list(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
) -> Result<ApiResponse<Vec<ApiKeyResponse>>, AppError> {
    let org_id = auth.organization.as_ref().map(|o| o.id);

    let state = state.read().await;
    let service = build_service(&state)?;
    let keys = service.list(org_id, auth.user.id).await?;

    Ok(ApiResponse::success(
        keys.into_iter()
            .map(|k| ApiKeyResponse {
                id: k.id,
                name: k.name,
                prefix: k.prefix,
                scopes: k.scopes,
                last_used_at: k.last_used_at,
                expires_at: k.expires_at,
                created_at: k.created_at,
            })
            .collect(),
    ))
}

pub async fn delete(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    _auth: AuthContext,
    Path(key_id): Path<Uuid>,
) -> Result<NoContent, AppError> {
    let state = state.read().await;
    let service = build_service(&state)?;
    service.delete(key_id).await?;
    Ok(NoContent)
}
