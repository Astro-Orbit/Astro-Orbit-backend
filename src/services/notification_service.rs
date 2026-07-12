use async_trait::async_trait;
use uuid::Uuid;

/// Notification service trait.
#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn list_notifications(&self, user_id: Uuid) -> Result<Vec<NotificationResult>, crate::AppError>;
    async fn mark_read(&self, notification_id: Uuid) -> Result<(), crate::AppError>;
    async fn update_preferences(&self, user_id: Uuid, email: bool, webhook: bool) -> Result<(), crate::AppError>;
}

pub struct NotificationResult {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
