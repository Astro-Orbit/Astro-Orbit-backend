use std::sync::Arc;

use axum::extract::Path;
use axum::{Extension, Json};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::context::AuthContext;
use crate::dto::user::*;
use crate::errors::AppError;
use crate::response::ApiResponse;
use crate::services::user_service::UserService;
use crate::state::AppState;

fn build_user_service(state: &AppState) -> Result<UserService, AppError> {
    let pool = state.db()?;
    Ok(UserService::new(
        pool.clone(),
        Arc::new(crate::repositories::user_repo::PgUserRepository::new(pool.clone())),
        Arc::new(crate::repositories::wallet_repo::PgWalletRepository::new(pool.clone())),
    ))
}

pub async fn get_me(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
) -> Result<ApiResponse<UserResponse>, AppError> {
    let state = state.read().await;
    let service = build_user_service(&state)?;
    let profile = service.get_user(auth.user.id).await?;

    Ok(ApiResponse::success(UserResponse {
        id: profile.id,
        stellar_public: profile.stellar_public,
        display_name: profile.display_name,
        avatar_url: profile.avatar_url,
        email: profile.email,
        email_verified: profile.email_verified,
        wallets: profile
            .wallets
            .into_iter()
            .map(|w| WalletDto { id: w.id, public_key: w.public_key, name: w.name, is_primary: w.is_primary })
            .collect(),
        created_at: profile.created_at,
    }))
}

pub async fn update_me(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Json(req): Json<UpdateUserRequest>,
) -> Result<ApiResponse<UserResponse>, AppError> {
    let state = state.read().await;
    let service = build_user_service(&state)?;
    let profile = service
        .update_user(auth.user.id, req.display_name.as_deref(), req.avatar_url.as_deref(), req.email.as_deref())
        .await?;

    Ok(ApiResponse::success(UserResponse {
        id: profile.id,
        stellar_public: profile.stellar_public,
        display_name: profile.display_name,
        avatar_url: profile.avatar_url,
        email: profile.email,
        email_verified: profile.email_verified,
        wallets: profile
            .wallets
            .into_iter()
            .map(|w| WalletDto { id: w.id, public_key: w.public_key, name: w.name, is_primary: w.is_primary })
            .collect(),
        created_at: profile.created_at,
    }))
}

pub async fn get_by_id(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<UserResponse>, AppError> {
    let state = state.read().await;
    let service = build_user_service(&state)?;
    let profile = service.get_user(id).await?;

    Ok(ApiResponse::success(UserResponse {
        id: profile.id,
        stellar_public: profile.stellar_public,
        display_name: profile.display_name,
        avatar_url: profile.avatar_url,
        email: profile.email,
        email_verified: profile.email_verified,
        wallets: profile
            .wallets
            .into_iter()
            .map(|w| WalletDto { id: w.id, public_key: w.public_key, name: w.name, is_primary: w.is_primary })
            .collect(),
        created_at: profile.created_at,
    }))
}
