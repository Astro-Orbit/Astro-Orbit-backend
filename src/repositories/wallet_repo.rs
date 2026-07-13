use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::wallet::Wallet;

#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn create(&self, public_key: &str) -> Result<Wallet, sqlx::Error>;
    async fn find_or_create(&self, public_key: &str, user_id: Uuid) -> Result<Wallet, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Wallet, sqlx::Error>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Wallet>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Wallet>, sqlx::Error>;
    async fn set_primary(&self, wallet_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct PgWalletRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgWalletRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WalletRepository for PgWalletRepository {
    async fn create(&self, public_key: &str) -> Result<Wallet, sqlx::Error> {
        sqlx::query_as::<_, Wallet>(
            r"
            INSERT INTO wallets (public_key)
            VALUES ($1)
            RETURNING id, public_key, name, user_id, is_primary, created_at, updated_at, deleted_at
            ",
        )
        .bind(public_key)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_or_create(&self, public_key: &str, user_id: Uuid) -> Result<Wallet, AppError> {
        let existing = sqlx::query_as::<_, Wallet>(
            r"
            SELECT id, public_key, name, user_id, is_primary, created_at, updated_at, deleted_at
            FROM wallets
            WHERE public_key = $1 AND deleted_at IS NULL
            ",
        )
        .bind(public_key)
        .fetch_optional(&*self.pool)
        .await
        .map_err(AppError::from)?;

        if let Some(wallet) = existing {
            return Ok(wallet);
        }

        let wallet = sqlx::query_as::<_, Wallet>(
            r"
            INSERT INTO wallets (public_key, user_id, is_primary)
            VALUES ($1, $2, NOT EXISTS (
                SELECT 1 FROM wallets WHERE user_id = $2 AND is_primary = true AND deleted_at IS NULL
            ))
            RETURNING id, public_key, name, user_id, is_primary, created_at, updated_at, deleted_at
            ",
        )
        .bind(public_key)
        .bind(user_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(wallet)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Wallet, sqlx::Error> {
        sqlx::query_as::<_, Wallet>(
            r"
            SELECT id, public_key, name, user_id, is_primary, created_at, updated_at, deleted_at
            FROM wallets
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Wallet>, sqlx::Error> {
        sqlx::query_as::<_, Wallet>(
            r"
            SELECT id, public_key, name, user_id, is_primary, created_at, updated_at, deleted_at
            FROM wallets
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY is_primary DESC, created_at ASC
            ",
        )
        .bind(user_id)
        .fetch_all(&*self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Wallet>, sqlx::Error> {
        sqlx::query_as::<_, Wallet>(
            r"
            SELECT id, public_key, name, user_id, is_primary, created_at, updated_at, deleted_at
            FROM wallets
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            ",
        )
        .fetch_all(&*self.pool)
        .await
    }

    async fn set_primary(&self, wallet_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
            UPDATE wallets SET is_primary = false WHERE user_id = $1;
            UPDATE wallets SET is_primary = true WHERE id = $2 AND user_id = $1;
            ",
        )
        .bind(user_id)
        .bind(wallet_id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r"UPDATE wallets SET deleted_at = NOW() WHERE id = $1").bind(id).execute(&*self.pool).await?;
        Ok(())
    }
}
