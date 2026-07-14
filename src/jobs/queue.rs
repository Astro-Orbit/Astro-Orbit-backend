use async_trait::async_trait;
use uuid::Uuid;

use crate::jobs::types::{DeadLetterJob, Job, JobPriority};

#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        priority: JobPriority,
        scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Job, sqlx::Error>;
    async fn dequeue(&self) -> Result<Option<Job>, sqlx::Error>;
    async fn mark_completed(&self, job_id: Uuid) -> Result<(), sqlx::Error>;
    async fn mark_failed(&self, job_id: Uuid, error: &str) -> Result<(), sqlx::Error>;
    async fn mark_retrying(&self, job_id: Uuid, error: &str) -> Result<(), sqlx::Error>;
    async fn send_to_dlq(&self, job_id: Uuid, error: &str) -> Result<DeadLetterJob, sqlx::Error>;
    async fn retry_dead_letters(&self) -> Result<usize, sqlx::Error>;
    async fn cleanup_completed(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<u64, sqlx::Error>;
}

pub struct PgJobQueue {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgJobQueue {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobQueue for PgJobQueue {
    async fn enqueue(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        priority: JobPriority,
        scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Job, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r"
            INSERT INTO job_queue (job_type, payload, priority, scheduled_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, job_type, payload, priority, status, scheduled_at,
                      retry_count, max_retries, last_error,
                      created_at, started_at, completed_at
            ",
        )
        .bind(job_type)
        .bind(payload)
        .bind(priority.as_i32())
        .bind(scheduled_at)
        .fetch_one(&*self.pool)
        .await
    }

    async fn dequeue(&self) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r"
            UPDATE job_queue
            SET status = 'running',
                started_at = NOW(),
                retry_count = retry_count + 1
            WHERE id = (
                SELECT id FROM job_queue
                WHERE status = 'pending'
                  AND (scheduled_at IS NULL OR scheduled_at <= NOW())
                ORDER BY priority DESC, created_at ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING id, job_type, payload, priority, status, scheduled_at,
                      retry_count, max_retries, last_error,
                      created_at, started_at, completed_at
            ",
        )
        .fetch_optional(&*self.pool)
        .await
    }

    async fn mark_completed(&self, job_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            UPDATE job_queue
            SET status = 'completed', completed_at = NOW()
            WHERE id = $1
            ",
        )
        .bind(job_id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn mark_failed(&self, job_id: Uuid, error: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            UPDATE job_queue
            SET status = 'failed', last_error = $2, completed_at = NOW()
            WHERE id = $1
            ",
        )
        .bind(job_id)
        .bind(error)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn mark_retrying(&self, job_id: Uuid, error: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            UPDATE job_queue
            SET status = 'retrying', last_error = $2
            WHERE id = $1
            ",
        )
        .bind(job_id)
        .bind(error)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn send_to_dlq(&self, job_id: Uuid, error: &str) -> Result<DeadLetterJob, sqlx::Error> {
        sqlx::query_as::<_, DeadLetterJob>(
            r"
            UPDATE job_queue
            SET status = 'dead_letter', last_error = $2, completed_at = NOW()
            WHERE id = $1
            RETURNING id, job_type, payload, priority, last_error, retry_count,
                      created_at, completed_at AS moved_to_dlq_at
            ",
        )
        .bind(job_id)
        .bind(error)
        .fetch_one(&*self.pool)
        .await
    }

    async fn retry_dead_letters(&self) -> Result<usize, sqlx::Error> {
        let result = sqlx::query(
            r"
            UPDATE job_queue
            SET status = 'pending', retry_count = 0, last_error = NULL, completed_at = NULL
            WHERE status = 'dead_letter'
              AND retry_count < max_retries
            ",
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected() as usize)
    }

    async fn cleanup_completed(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r"
            DELETE FROM job_queue
            WHERE status IN ('completed', 'dead_letter')
              AND completed_at < $1
            ",
        )
        .bind(older_than)
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
