use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::audit_log::AuditLog;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait AuditLogRepository: Send + Sync {
    async fn insert(
        &self,
        org_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Uuid,
        metadata: &serde_json::Value,
        ip_address: Option<&str>,
    ) -> Result<AuditLog, AppError>;
    async fn find_by_org(&self, org_id: Uuid, limit: i64, offset: i64) -> Result<Vec<AuditLog>, AppError>;
}

pub struct PgAuditLogRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgAuditLogRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogRepository for PgAuditLogRepository {
    async fn insert(
        &self,
        org_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Uuid,
        metadata: &serde_json::Value,
        ip_address: Option<&str>,
    ) -> Result<AuditLog, AppError> {
        sqlx::query_as::<_, AuditLog>(
            r"
            INSERT INTO audit_logs (organization_id, actor_id, action, resource_type, resource_id, metadata, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6, $7::inet)
            RETURNING id, organization_id, actor_id, action, resource_type, resource_id,
                      metadata, ip_address::text, created_at
            ",
        )
        .bind(org_id)
        .bind(actor_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(metadata)
        .bind(ip_address)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_org(&self, org_id: Uuid, limit: i64, offset: i64) -> Result<Vec<AuditLog>, AppError> {
        sqlx::query_as::<_, AuditLog>(
            r"
            SELECT id, organization_id, actor_id, action, resource_type, resource_id,
                   metadata, ip_address::text, created_at
            FROM audit_logs
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            ",
        )
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await
        .map_err(AppError::from)
    }
}
