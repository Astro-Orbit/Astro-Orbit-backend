use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait DeploymentRepository: Send + Sync {
    async fn create(&self, project_id: Uuid, environment: &str, created_by: Uuid) -> Result<crate::models::deployment::Deployment, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<crate::models::deployment::Deployment, sqlx::Error>;
    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<crate::models::deployment::Deployment>, sqlx::Error>;
}

pub struct PgDeploymentRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgDeploymentRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeploymentRepository for PgDeploymentRepository {
    async fn create(&self, project_id: Uuid, environment: &str, created_by: Uuid) -> Result<crate::models::deployment::Deployment, sqlx::Error> {
        sqlx::query_as::<_, crate::models::deployment::Deployment>(
            r#"
            INSERT INTO deployments (project_id, environment, created_by)
            VALUES ($1, $2, $3)
            RETURNING id, project_id, environment, status, commit_sha, commit_message,
                      branch, version, metadata, created_by, started_at, completed_at,
                      created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(environment)
        .bind(created_by)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<crate::models::deployment::Deployment, sqlx::Error> {
        sqlx::query_as::<_, crate::models::deployment::Deployment>(
            r#"
            SELECT id, project_id, environment, status, commit_sha, commit_message,
                   branch, version, metadata, created_by, started_at, completed_at,
                   created_at, updated_at
            FROM deployments
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<crate::models::deployment::Deployment>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::deployment::Deployment>(
            r#"
            SELECT id, project_id, environment, status, commit_sha, commit_message,
                   branch, version, metadata, created_by, started_at, completed_at,
                   created_at, updated_at
            FROM deployments
            WHERE project_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(&*self.pool)
        .await
    }
}
