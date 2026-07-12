use async_trait::async_trait;
use uuid::Uuid;

use crate::models::project::Project;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, org_id: Uuid, name: &str, slug: &str) -> Result<Project, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Project, sqlx::Error>;
    async fn find_by_org(&self, org_id: Uuid) -> Result<Vec<Project>, sqlx::Error>;
    async fn update(&self, id: Uuid, name: Option<&str>) -> Result<Project, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct PgProjectRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgProjectRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn create(&self, org_id: Uuid, name: &str, slug: &str) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            INSERT INTO projects (organization_id, name, slug)
            VALUES ($1, $2, $3)
            RETURNING id, organization_id, name, slug, description, visibility, network,
                      settings, created_at, updated_at, deleted_at
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(slug)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            SELECT id, organization_id, name, slug, description, visibility, network,
                   settings, created_at, updated_at, deleted_at
            FROM projects
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_org(&self, org_id: Uuid) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            SELECT id, organization_id, name, slug, description, visibility, network,
                   settings, created_at, updated_at, deleted_at
            FROM projects
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&*self.pool)
        .await
    }

    async fn update(&self, id: Uuid, name: Option<&str>) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            UPDATE projects
            SET name = COALESCE($2, name), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, organization_id, name, slug, description, visibility, network,
                      settings, created_at, updated_at, deleted_at
            "#,
        )
        .bind(id)
        .bind(name)
        .fetch_one(&*self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE projects SET deleted_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
