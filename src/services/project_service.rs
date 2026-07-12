use async_trait::async_trait;
use uuid::Uuid;

/// Project service trait.
#[async_trait]
pub trait ProjectService: Send + Sync {
    async fn create_project(&self, org_id: Uuid, name: &str, slug: &str) -> Result<ProjectResult, crate::AppError>;
    async fn get_project(&self, project_id: Uuid) -> Result<ProjectResult, crate::AppError>;
    async fn list_projects(&self, org_id: Uuid) -> Result<Vec<ProjectResult>, crate::AppError>;
    async fn update_project(&self, project_id: Uuid, name: Option<&str>) -> Result<ProjectResult, crate::AppError>;
    async fn delete_project(&self, project_id: Uuid) -> Result<(), crate::AppError>;
}

pub struct ProjectResult {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
    pub network: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
