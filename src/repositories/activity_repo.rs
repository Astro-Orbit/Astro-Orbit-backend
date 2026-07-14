use async_trait::async_trait;
use uuid::Uuid;

use crate::models::activity::Activity;

#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn record(
        &self,
        org_id: Uuid,
        actor_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: Uuid,
        metadata: serde_json::Value,
    ) -> Result<Activity, sqlx::Error>;
    async fn find_by_org(&self, org_id: Uuid, limit: i64) -> Result<Vec<Activity>, sqlx::Error>;
    async fn find_by_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<Activity>, sqlx::Error>;
    async fn count_by_org_since(&self, org_id: Uuid, since: chrono::DateTime<chrono::Utc>) -> Result<i64, sqlx::Error>;
}

pub struct PgActivityRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgActivityRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActivityRepository for PgActivityRepository {
    async fn record(
        &self,
        org_id: Uuid,
        actor_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: Uuid,
        metadata: serde_json::Value,
    ) -> Result<Activity, sqlx::Error> {
        sqlx::query_as::<_, Activity>(
            r"
            INSERT INTO activity_log (organization_id, actor_id, action, resource_type, resource_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, actor_id, action, resource_type, resource_id, metadata, created_at
            ",
        )
        .bind(org_id)
        .bind(actor_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(metadata)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_org(&self, org_id: Uuid, limit: i64) -> Result<Vec<Activity>, sqlx::Error> {
        sqlx::query_as::<_, Activity>(
            r"
            SELECT id, organization_id, actor_id, action, resource_type, resource_id, metadata, created_at
            FROM activity_log
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            ",
        )
        .bind(org_id)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await
    }

    async fn find_by_user(&self, user_id: Uuid, limit: i64) -> Result<Vec<Activity>, sqlx::Error> {
        sqlx::query_as::<_, Activity>(
            r"
            SELECT a.id, a.organization_id, a.actor_id, a.action, a.resource_type, a.resource_id, a.metadata, a.created_at
            FROM activity_log a
            JOIN organization_members om ON om.organization_id = a.organization_id
            WHERE om.user_id = $1
            ORDER BY a.created_at DESC
            LIMIT $2
            ",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await
    }

    async fn count_by_org_since(&self, org_id: Uuid, since: chrono::DateTime<chrono::Utc>) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r"
            SELECT COUNT(*) FROM activity_log
            WHERE organization_id = $1 AND created_at >= $2
            ",
        )
        .bind(org_id)
        .bind(since)
        .fetch_one(&*self.pool)
        .await
    }
}
