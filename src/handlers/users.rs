use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::ApiResponse;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub stellar_public: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn get_me() -> ApiResponse<UserResponse> {
    ApiResponse::success(UserResponse {
        id: Uuid::new_v4(),
        stellar_public: String::new(),
        display_name: None,
        email: None,
        avatar_url: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn update_me(Json(_req): Json<UpdateUserRequest>) -> ApiResponse<UserResponse> {
    ApiResponse::success(UserResponse {
        id: Uuid::new_v4(),
        stellar_public: String::new(),
        display_name: None,
        email: None,
        avatar_url: None,
        created_at: chrono::Utc::now(),
    })
}

pub async fn get_by_id(Path(_id): Path<Uuid>) -> ApiResponse<UserResponse> {
    ApiResponse::success(UserResponse {
        id: Uuid::new_v4(),
        stellar_public: String::new(),
        display_name: None,
        email: None,
        avatar_url: None,
        created_at: chrono::Utc::now(),
    })
}
