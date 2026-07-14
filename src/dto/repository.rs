use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub url: String,
    pub provider: String,
    pub branch: Option<String>,
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRepositoryRequest {
    pub name: Option<String>,
    pub branch: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RepositoryResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub url: String,
    pub provider: String,
    pub branch: String,
    pub is_active: bool,
    pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct SyncRepositoryResponse {
    pub id: Uuid,
    pub status: String,
    pub last_synced_at: chrono::DateTime<chrono::Utc>,
}
