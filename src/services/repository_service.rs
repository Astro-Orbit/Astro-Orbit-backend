use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait RepositoryService: Send + Sync {
    async fn link_repository(
        &self,
        project_id: Uuid,
        name: &str,
        url: &str,
        provider: &str,
        branch: &str,
        webhook_secret: Option<&str>,
    ) -> Result<RepositoryResult, crate::AppError>;
    async fn get_repository(&self, id: Uuid) -> Result<RepositoryResult, crate::AppError>;
    async fn list_repositories(&self, project_id: Uuid) -> Result<Vec<RepositoryResult>, crate::AppError>;
    async fn update_repository(
        &self,
        id: Uuid,
        name: Option<&str>,
        branch: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<RepositoryResult, crate::AppError>;
    async fn unlink_repository(&self, id: Uuid) -> Result<(), crate::AppError>;
    async fn sync_repository(&self, id: Uuid) -> Result<RepositoryResult, crate::AppError>;
}

pub struct RepositoryResult {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub url: String,
    pub provider: String,
    pub branch: String,
    pub is_active: bool,
    pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct RepositoryServiceImpl {
    repo: std::sync::Arc<dyn crate::repositories::repository_repo::RepositoryRepository>,
}

impl RepositoryServiceImpl {
    #[must_use]
    pub fn new(
        _pool: std::sync::Arc<sqlx::PgPool>,
        repo: std::sync::Arc<dyn crate::repositories::repository_repo::RepositoryRepository>,
    ) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl RepositoryService for RepositoryServiceImpl {
    async fn link_repository(
        &self,
        project_id: Uuid,
        name: &str,
        url: &str,
        provider: &str,
        branch: &str,
        webhook_secret: Option<&str>,
    ) -> Result<RepositoryResult, crate::AppError> {
        let model = self.repo.create(project_id, name, url, provider, branch, webhook_secret).await?;
        Ok(RepositoryResult {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            url: model.url,
            provider: model.provider,
            branch: model.branch,
            is_active: model.is_active,
            last_synced_at: model.last_synced_at,
            created_at: model.created_at,
        })
    }

    async fn get_repository(&self, id: Uuid) -> Result<RepositoryResult, crate::AppError> {
        let model = self.repo.find_by_id(id).await?;
        Ok(RepositoryResult {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            url: model.url,
            provider: model.provider,
            branch: model.branch,
            is_active: model.is_active,
            last_synced_at: model.last_synced_at,
            created_at: model.created_at,
        })
    }

    async fn list_repositories(&self, project_id: Uuid) -> Result<Vec<RepositoryResult>, crate::AppError> {
        let models = self.repo.find_by_project(project_id).await?;
        Ok(models
            .into_iter()
            .map(|m| RepositoryResult {
                id: m.id,
                project_id: m.project_id,
                name: m.name,
                url: m.url,
                provider: m.provider,
                branch: m.branch,
                is_active: m.is_active,
                last_synced_at: m.last_synced_at,
                created_at: m.created_at,
            })
            .collect())
    }

    async fn update_repository(
        &self,
        id: Uuid,
        name: Option<&str>,
        branch: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<RepositoryResult, crate::AppError> {
        let model = self.repo.update(id, name, branch, is_active).await?;
        Ok(RepositoryResult {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            url: model.url,
            provider: model.provider,
            branch: model.branch,
            is_active: model.is_active,
            last_synced_at: model.last_synced_at,
            created_at: model.created_at,
        })
    }

    async fn unlink_repository(&self, id: Uuid) -> Result<(), crate::AppError> {
        self.repo.delete(id).await?;
        Ok(())
    }

    async fn sync_repository(&self, id: Uuid) -> Result<RepositoryResult, crate::AppError> {
        let model = self.repo.update_synced_at(id).await?;
        Ok(RepositoryResult {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            url: model.url,
            provider: model.provider,
            branch: model.branch,
            is_active: model.is_active,
            last_synced_at: model.last_synced_at,
            created_at: model.created_at,
        })
    }
}
