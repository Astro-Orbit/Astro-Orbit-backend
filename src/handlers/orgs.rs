use std::sync::Arc;

use axum::extract::Path;
use axum::{Extension, Json};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::context::AuthContext;
use crate::dto::org::*;
use crate::errors::AppError;
use crate::response::{ApiResponse, CreatedResponse, NoContent};
use crate::services::org_service::OrgService;
use crate::state::AppState;

fn build_org_service(state: &AppState) -> Result<OrgService, AppError> {
    let pool = state.db()?;
    Ok(OrgService::new(
        pool.clone(),
        Arc::new(crate::repositories::org_repo::PgOrgRepository::new(pool.clone())),
        Arc::new(crate::repositories::user_repo::PgUserRepository::new(pool.clone())),
        Arc::new(crate::repositories::invitation_repo::PgInvitationRepository::new(pool.clone())),
    ))
}

pub async fn create(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Json(req): Json<CreateOrgRequest>,
) -> Result<CreatedResponse<OrgResponse>, AppError> {
    let state = state.read().await;
    let service = build_org_service(&state)?;
    let org = service.create(&req.name, &req.slug, req.description.as_deref(), auth.user.id).await?;

    Ok(CreatedResponse::new(OrgResponse {
        id: org.id,
        name: org.name,
        slug: org.slug,
        description: org.description,
        role: org.role,
        member_count: org.member_count,
        created_at: org.created_at,
    }))
}

pub async fn list(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
) -> Result<ApiResponse<Vec<OrgResponse>>, AppError> {
    let state = state.read().await;
    let service = build_org_service(&state)?;
    let orgs = service.list(auth.user.id).await?;

    Ok(ApiResponse::success(
        orgs.into_iter()
            .map(|o| OrgResponse {
                id: o.id,
                name: o.name,
                slug: o.slug,
                description: o.description,
                role: o.role,
                member_count: o.member_count,
                created_at: o.created_at,
            })
            .collect(),
    ))
}

pub async fn get_by_id(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<OrgResponse>, AppError> {
    let state = state.read().await;
    let service = build_org_service(&state)?;
    let org = service.get(id, auth.user.id).await?;

    Ok(ApiResponse::success(OrgResponse {
        id: org.id,
        name: org.name,
        slug: org.slug,
        description: org.description,
        role: org.role,
        member_count: org.member_count,
        created_at: org.created_at,
    }))
}

pub async fn update(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateOrgRequest>,
) -> Result<ApiResponse<OrgResponse>, AppError> {
    let role = auth.role().copied().unwrap_or(crate::permissions::role::Role::Viewer);
    crate::permissions::PolicyEngine::check(&role, "org:update")?;

    let state = state.read().await;
    let service = build_org_service(&state)?;
    let org = service.update(id, req.name.as_deref(), req.description.as_deref()).await?;

    Ok(ApiResponse::success(OrgResponse {
        id: org.id,
        name: org.name,
        slug: org.slug,
        description: org.description,
        role: org.role,
        member_count: org.member_count,
        created_at: org.created_at,
    }))
}

pub async fn delete(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<NoContent, AppError> {
    let role = auth.role().copied().unwrap_or(crate::permissions::role::Role::Viewer);
    crate::permissions::PolicyEngine::check(&role, "org:delete")?;

    let state = state.read().await;
    let service = build_org_service(&state)?;
    service.delete(id).await?;
    Ok(NoContent)
}

pub async fn invite(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Path(org_id): Path<Uuid>,
    Json(req): Json<InviteRequest>,
) -> Result<CreatedResponse<InviteResponse>, AppError> {
    let role = auth.role().copied().unwrap_or(crate::permissions::role::Role::Viewer);
    crate::permissions::PolicyEngine::check(&role, "org:members:invite")?;

    let state = state.read().await;
    let service = build_org_service(&state)?;
    let result = service.invite(org_id, auth.user.id, &req.email, &req.role).await?;

    Ok(CreatedResponse::new(InviteResponse {
        id: result.id,
        token: result.token,
        organization_id: result.organization_id,
        email: result.email,
        role: result.role,
        expires_at: result.expires_at,
    }))
}

pub async fn accept_invite(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    auth: AuthContext,
    Path(token): Path<String>,
) -> Result<ApiResponse<OrgResponse>, AppError> {
    let state = state.read().await;
    let service = build_org_service(&state)?;
    let org = service.accept_invite(&token, auth.user.id).await?;

    Ok(ApiResponse::success(OrgResponse {
        id: org.id,
        name: org.name,
        slug: org.slug,
        description: org.description,
        role: org.role,
        member_count: org.member_count,
        created_at: org.created_at,
    }))
}

pub async fn reject_invite(
    Extension(state): Extension<Arc<RwLock<AppState>>>,
    Path(token): Path<String>,
) -> Result<NoContent, AppError> {
    let state = state.read().await;
    let service = build_org_service(&state)?;
    service.reject_invite(&token).await?;
    Ok(NoContent)
}

pub async fn add_member(
    Path((_org_id, _user_id)): Path<(Uuid, Uuid)>,
    Json(_req): Json<AddMemberRequest>,
) -> CreatedResponse<MemberResponse> {
    CreatedResponse::new(MemberResponse { user_id: Uuid::new_v4(), role: String::new(), joined_at: chrono::Utc::now() })
}

pub async fn list_members(Path(_org_id): Path<Uuid>) -> ApiResponse<Vec<MemberResponse>> {
    ApiResponse::success(vec![])
}

pub async fn update_member(
    Path((_org_id, _user_id)): Path<(Uuid, Uuid)>,
    Json(_req): Json<UpdateMemberRequest>,
) -> ApiResponse<MemberResponse> {
    ApiResponse::success(MemberResponse { user_id: Uuid::new_v4(), role: String::new(), joined_at: chrono::Utc::now() })
}

pub async fn remove_member(Path((_org_id, _user_id)): Path<(Uuid, Uuid)>) -> NoContent {
    NoContent
}

pub async fn list_roles(Path(_org_id): Path<Uuid>) -> ApiResponse<Vec<crate::dto::org::MemberResponse>> {
    ApiResponse::success(vec![])
}

pub async fn create_role(
    Path(_org_id): Path<Uuid>,
    Json(_req): Json<crate::dto::org::AddMemberRequest>,
) -> CreatedResponse<crate::dto::org::MemberResponse> {
    CreatedResponse::new(crate::dto::org::MemberResponse {
        user_id: Uuid::new_v4(),
        role: String::new(),
        joined_at: chrono::Utc::now(),
    })
}
