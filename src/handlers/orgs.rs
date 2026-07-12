use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, CreatedResponse, NoContent};

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

pub async fn create(Json(_req): Json<CreateOrgRequest>) -> CreatedResponse<OrgResponse> {
    CreatedResponse::new(OrgResponse {
        id: Uuid::new_v4(),
        name: String::new(),
        slug: String::new(),
        description: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn list() -> ApiResponse<Vec<OrgResponse>> {
    ApiResponse::success(vec![])
}

pub async fn get_by_id(Path(_id): Path<Uuid>) -> ApiResponse<OrgResponse> {
    ApiResponse::success(OrgResponse {
        id: Uuid::new_v4(),
        name: String::new(),
        slug: String::new(),
        description: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn update(
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateOrgRequest>,
) -> ApiResponse<OrgResponse> {
    ApiResponse::success(OrgResponse {
        id: Uuid::new_v4(),
        name: String::new(),
        slug: String::new(),
        description: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn delete(Path(_id): Path<Uuid>) -> NoContent {
    NoContent
}

pub async fn add_member(
    Path(_org_id): Path<Uuid>,
    Json(_req): Json<AddMemberRequest>,
) -> CreatedResponse<MemberResponse> {
    CreatedResponse::new(MemberResponse {
        user_id: Uuid::new_v4(),
        role: String::new(),
        joined_at: chrono::Utc::now(),
    })
}

pub async fn list_members(Path(_org_id): Path<Uuid>) -> ApiResponse<Vec<MemberResponse>> {
    ApiResponse::success(vec![])
}

pub async fn update_member(
    Path((_org_id, _user_id)): Path<(Uuid, Uuid)>,
    Json(_req): Json<UpdateMemberRequest>,
) -> ApiResponse<MemberResponse> {
    ApiResponse::success(MemberResponse {
        user_id: Uuid::new_v4(),
        role: String::new(),
        joined_at: chrono::Utc::now(),
    })
}

pub async fn remove_member(
    Path((_org_id, _user_id)): Path<(Uuid, Uuid)>,
) -> NoContent {
    NoContent
}

pub async fn list_roles(Path(_org_id): Path<Uuid>) -> ApiResponse<Vec<RoleResponse>> {
    ApiResponse::success(vec![])
}

pub async fn create_role(
    Path(_org_id): Path<Uuid>,
    Json(_req): Json<CreateRoleRequest>,
) -> CreatedResponse<RoleResponse> {
    CreatedResponse::new(RoleResponse {
        id: Uuid::new_v4(),
        name: String::new(),
        permissions: vec![],
    })
}
