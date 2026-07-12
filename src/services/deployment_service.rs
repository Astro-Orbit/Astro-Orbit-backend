use async_trait::async_trait;
use uuid::Uuid;

/// Deployment service trait.
#[async_trait]
pub trait DeploymentService: Send + Sync {
    async fn create_deployment(&self, project_id: Uuid, environment: &str) -> Result<DeploymentResult, crate::AppError>;
    async fn get_deployment(&self, deployment_id: Uuid) -> Result<DeploymentResult, crate::AppError>;
    async fn list_deployments(&self, project_id: Uuid) -> Result<Vec<DeploymentResult>, crate::AppError>;
    async fn rollback_deployment(&self, deployment_id: Uuid) -> Result<DeploymentResult, crate::AppError>;
    async fn cancel_deployment(&self, deployment_id: Uuid) -> Result<(), crate::AppError>;
}

pub struct DeploymentResult {
    pub id: Uuid,
    pub project_id: Uuid,
    pub environment: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
