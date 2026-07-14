use async_trait::async_trait;
use uuid::Uuid;

use crate::models::repository::Repository;

#[async_trait]
pub trait RepositoryRepository: Send + Sync {
    async fn create(
        &self,
        project_id: Uuid,
        name: &str,
        url: &str,
        provider: &str,
        branch: &str,
        webhook_secret: Option<&str>,
    ) -> Result<Repository, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Repository, sqlx::Error>;
    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Repository>, sqlx::Error>;
    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        branch: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Repository, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn update_synced_at(&self, id: Uuid) -> Result<Repository, sqlx::Error>;
}

pub struct PgRepositoryRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgRepositoryRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RepositoryRepository for PgRepositoryRepository {
    async fn create(
        &self,
        project_id: Uuid,
        name: &str,
        url: &str,
        provider: &str,
        branch: &str,
        webhook_secret: Option<&str>,
    ) -> Result<Repository, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            r"
            INSERT INTO repositories (project_id, name, url, provider, branch, webhook_secret)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, project_id, name, url, provider, branch, webhook_secret,
                      is_active, last_synced_at, created_at, updated_at, deleted_at
            ",
        )
        .bind(project_id)
        .bind(name)
        .bind(url)
        .bind(provider)
        .bind(branch)
        .bind(webhook_secret)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Repository, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            r"
            SELECT id, project_id, name, url, provider, branch, webhook_secret,
                   is_active, last_synced_at, created_at, updated_at, deleted_at
            FROM repositories
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<Repository>, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            r"
            SELECT id, project_id, name, url, provider, branch, webhook_secret,
                   is_active, last_synced_at, created_at, updated_at, deleted_at
            FROM repositories
            WHERE project_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            ",
        )
        .bind(project_id)
        .fetch_all(&*self.pool)
        .await
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        branch: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Repository, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            r"
            UPDATE repositories
            SET
                name = COALESCE($2, name),
                branch = COALESCE($3, branch),
                is_active = COALESCE($4, is_active),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, project_id, name, url, provider, branch, webhook_secret,
                      is_active, last_synced_at, created_at, updated_at, deleted_at
            ",
        )
        .bind(id)
        .bind(name)
        .bind(branch)
        .bind(is_active)
        .fetch_one(&*self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r"UPDATE repositories SET deleted_at = NOW() WHERE id = $1").bind(id).execute(&*self.pool).await?;
        Ok(())
    }

    async fn update_synced_at(&self, id: Uuid) -> Result<Repository, sqlx::Error> {
        sqlx::query_as::<_, Repository>(
            r"
            UPDATE repositories
            SET last_synced_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, project_id, name, url, provider, branch, webhook_secret,
                      is_active, last_synced_at, created_at, updated_at, deleted_at
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }
}
