use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
