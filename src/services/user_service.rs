use async_trait::async_trait;
use uuid::Uuid;

/// User service trait.
#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_user(&self, user_id: Uuid) -> Result<UserResult, crate::AppError>;
    async fn update_user(&self, user_id: Uuid, display_name: Option<&str>) -> Result<UserResult, crate::AppError>;
}

pub struct UserResult {
    pub id: Uuid,
    pub stellar_public: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
