use std::sync::Arc;

use crate::auth::context::AuthContext;
use crate::dto::auth::*;
use crate::errors::AppError;
use crate::response::{ApiResponse, CreatedResponse, NoContent};
use crate::services::auth_service::AuthService;
use crate::state::AppState;
use crate::validation::stellar::validate_public_key;
use axum::Extension;
use axum::Json;
use tokio::sync::RwLock;

fn build_auth_service(state: &AppState) -> Result<AuthService, AppError> {
    let pool = state.db()?;
    let cache = state.cache()?;
    let config = state.config.clone();
    Ok(AuthService::new(
        pool.clone(),
        cache,
        config,
        Arc::new(crate::repositories::user_repo::PgUserRepository::new(pool.clone())),
        Arc::new(crate::repositories::wallet_repo::PgWalletRepository::new(pool.clone())),
        Arc::new(crate::repositories::challenge_repo::PgChallengeRepository::new(pool.clone())),
    ))
}

pub async fn challenge(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(req): Json<ChallengeRequest>,
) -> Result<CreatedResponse<ChallengeResponse>, AppError> {
    if !validate_public_key(&req.public_key) {
        return Err(AppError::bad_request("invalid Stellar public key format"));
    }

    let state = state.read().await;
    let auth_service = build_auth_service(&state)?;
    let (challenge, challenge_id) = auth_service.generate_challenge(&req.public_key).await?;

    Ok(CreatedResponse::new(ChallengeResponse { challenge, challenge_id }))
}

pub async fn login(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(req): Json<LoginRequest>,
) -> Result<CreatedResponse<LoginResponse>, AppError> {
    let state = state.read().await;
    let auth_service = build_auth_service(&state)?;
    let result = auth_service.verify_and_login(&req.public_key, &req.signature, req.challenge_id).await?;

    Ok(CreatedResponse::new(LoginResponse {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        user: UserDto { id: result.user_id, stellar_public: result.public_key, display_name: result.display_name },
    }))
}

pub async fn refresh(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Json(req): Json<RefreshRequest>,
) -> Result<ApiResponse<RefreshResponse>, AppError> {
    let state = state.read().await;
    let auth_service = build_auth_service(&state)?;
    let result = auth_service.refresh_session(&req.refresh_token).await?;

    Ok(ApiResponse::success(RefreshResponse { access_token: result.access_token, refresh_token: result.refresh_token }))
}

pub async fn logout(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
) -> Result<NoContent, AppError> {
    let state = state.read().await;
    let auth_service = build_auth_service(&state)?;
    auth_service.logout(auth.session.id).await?;
    Ok(NoContent)
}

pub async fn logout_all(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
) -> Result<ApiResponse<serde_json::Value>, AppError> {
    let state = state.read().await;
    let auth_service = build_auth_service(&state)?;
    let count = auth_service.logout_all(auth.user.id).await?;
    Ok(ApiResponse::success(serde_json::json!({ "revoked_sessions": count })))
}
