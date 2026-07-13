use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::challenge::WalletChallenge;

#[async_trait]
pub trait ChallengeRepository: Send + Sync {
    async fn create(
        &self,
        public_key: &str,
        challenge: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<WalletChallenge, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<WalletChallenge, AppError>;
    async fn mark_used(&self, id: Uuid) -> Result<(), AppError>;
    async fn delete_expired(&self) -> Result<u64, AppError>;
}

pub struct PgChallengeRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgChallengeRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChallengeRepository for PgChallengeRepository {
    async fn create(
        &self,
        public_key: &str,
        challenge: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<WalletChallenge, AppError> {
        sqlx::query_as::<_, WalletChallenge>(
            r"
            INSERT INTO wallet_challenges (public_key, challenge, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, public_key, challenge, expires_at, used_at, created_at
            ",
        )
        .bind(public_key)
        .bind(challenge)
        .bind(expires_at)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<WalletChallenge, AppError> {
        sqlx::query_as::<_, WalletChallenge>(
            r"
            SELECT id, public_key, challenge, expires_at, used_at, created_at
            FROM wallet_challenges
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)
    }

    async fn mark_used(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE wallet_challenges
            SET used_at = NOW()
            WHERE id = $1 AND used_at IS NULL
            ",
        )
        .bind(id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query(
            r"
            DELETE FROM wallet_challenges
            WHERE expires_at < NOW()
            ",
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
