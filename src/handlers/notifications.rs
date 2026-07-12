use axum::extract::Path;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::response::{ApiResponse, NoContent};

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationPreferencesRequest {
    pub email: bool,
    pub webhook: bool,
    pub in_app: bool,
}

pub async fn list() -> ApiResponse<Vec<NotificationResponse>> {
    ApiResponse::success(vec![])
}

pub async fn mark_read(Path(_id): Path<Uuid>) -> NoContent {
    NoContent
}

pub async fn preferences(
    Json(_req): Json<NotificationPreferencesRequest>,
) -> ApiResponse<serde_json::Value> {
    ApiResponse::success(serde_json::json!({}))
}
