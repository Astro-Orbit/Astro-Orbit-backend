use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::api_key::ApiKey;

/// NOTE: This repository provides an extended interface beyond the
/// existing stub API routes. The `api_keys` table already exists in
/// migration 001 with org-level keys. This adds user-level key support.
#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait ApiKeyRepository: Send + Sync {
    async fn create(
        &self,
        org_id: Option<Uuid>,
        user_id: Uuid,
        name: &str,
        key_hash: &str,
        key_prefix: &str,
        scopes: &[String],
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<ApiKey, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<ApiKey, AppError>;
    async fn find_by_org(&self, org_id: Uuid) -> Result<Vec<ApiKey>, AppError>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>, AppError>;
    async fn find_by_hash(&self, key_hash: &str) -> Result<ApiKey, AppError>;
    async fn update_last_used(&self, id: Uuid) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct PgApiKeyRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgApiKeyRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn create(
        &self,
        org_id: Option<Uuid>,
        user_id: Uuid,
        name: &str,
        key_hash: &str,
        key_prefix: &str,
        scopes: &[String],
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<ApiKey, AppError> {
        sqlx::query_as::<_, ApiKey>(
            r"
            INSERT INTO api_keys (organization_id, user_id, name, key_hash, key_prefix, permissions, expires_at, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $2)
            RETURNING id, organization_id, user_id, name, key_hash, key_prefix,
                      permissions, last_used_at, expires_at, created_by, created_at, deleted_at
            ",
        )
        .bind(org_id)
        .bind(user_id)
        .bind(name)
        .bind(key_hash)
        .bind(key_prefix)
        .bind(serde_json::to_value(scopes).unwrap_or_default())
        .bind(expires_at)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<ApiKey, AppError> {
        sqlx::query_as::<_, ApiKey>(
            r"
            SELECT id, organization_id, user_id, name, key_hash, key_prefix,
                   permissions, last_used_at, expires_at, created_by, created_at, deleted_at
            FROM api_keys
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_org(&self, org_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        sqlx::query_as::<_, ApiKey>(
            r"
            SELECT id, organization_id, user_id, name, key_hash, key_prefix,
                   permissions, last_used_at, expires_at, created_by, created_at, deleted_at
            FROM api_keys
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            ",
        )
        .bind(org_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        sqlx::query_as::<_, ApiKey>(
            r"
            SELECT id, organization_id, user_id, name, key_hash, key_prefix,
                   permissions, last_used_at, expires_at, created_by, created_at, deleted_at
            FROM api_keys
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            ",
        )
        .bind(user_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_hash(&self, key_hash: &str) -> Result<ApiKey, AppError> {
        sqlx::query_as::<_, ApiKey>(
            r"
            SELECT id, organization_id, user_id, name, key_hash, key_prefix,
                   permissions, last_used_at, expires_at, created_by, created_at, deleted_at
            FROM api_keys
            WHERE key_hash = $1 AND deleted_at IS NULL
            ",
        )
        .bind(key_hash)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn update_last_used(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE api_keys SET last_used_at = NOW() WHERE id = $1
            ",
        )
        .bind(id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE api_keys SET deleted_at = NOW() WHERE id = $1
            ",
        )
        .bind(id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
