use async_trait::async_trait;
use uuid::Uuid;

use crate::models::user::User;

/// User repository trait.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, stellar_public: &str) -> Result<User, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<User, sqlx::Error>;
    async fn find_by_stellar_public(&self, stellar_public: &str) -> Result<User, sqlx::Error>;
    async fn update(&self, id: Uuid, display_name: Option<&str>) -> Result<User, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

/// `PostgreSQL` implementation of `UserRepository`.
pub struct PgUserRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgUserRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, stellar_public: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r"
            INSERT INTO users (stellar_public)
            VALUES ($1)
            RETURNING id, stellar_public, display_name, avatar_url, email, email_verified,
                      totp_enabled, created_at, updated_at, deleted_at
            ",
        )
        .bind(stellar_public)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r"
            SELECT id, stellar_public, display_name, avatar_url, email, email_verified,
                   totp_enabled, created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_stellar_public(&self, stellar_public: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r"
            SELECT id, stellar_public, display_name, avatar_url, email, email_verified,
                   totp_enabled, created_at, updated_at, deleted_at
            FROM users
            WHERE stellar_public = $1 AND deleted_at IS NULL
            ",
        )
        .bind(stellar_public)
        .fetch_one(&*self.pool)
        .await
    }

    async fn update(&self, id: Uuid, display_name: Option<&str>) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r"
            UPDATE users
            SET display_name = COALESCE($2, display_name),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, stellar_public, display_name, avatar_url, email, email_verified,
                      totp_enabled, created_at, updated_at, deleted_at
            ",
        )
        .bind(id)
        .bind(display_name)
        .fetch_one(&*self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            UPDATE users SET deleted_at = NOW() WHERE id = $1
            ",
        )
        .bind(id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
