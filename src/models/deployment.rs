use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Deployment {
    pub id: Uuid,
    pub project_id: Uuid,
    pub environment: String,
    pub status: String,
    pub commit_sha: Option<String>,
    pub commit_message: Option<String>,
    pub branch: Option<String>,
    pub version: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
