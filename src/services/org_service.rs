use async_trait::async_trait;
use uuid::Uuid;

/// Organization service trait.
#[async_trait]
pub trait OrgService: Send + Sync {
    async fn create_org(&self, name: &str, slug: &str, description: Option<&str>, owner_id: Uuid) -> Result<OrgResult, crate::AppError>;
    async fn get_org(&self, org_id: Uuid) -> Result<OrgResult, crate::AppError>;
    async fn list_orgs(&self, user_id: Uuid) -> Result<Vec<OrgResult>, crate::AppError>;
    async fn update_org(&self, org_id: Uuid, name: Option<&str>, description: Option<&str>) -> Result<OrgResult, crate::AppError>;
    async fn delete_org(&self, org_id: Uuid) -> Result<(), crate::AppError>;
}

pub struct OrgResult {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
