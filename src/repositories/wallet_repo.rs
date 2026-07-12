use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn create(&self, public_key: &str) -> Result<crate::models::wallet::Wallet, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<crate::models::wallet::Wallet, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<crate::models::wallet::Wallet>, sqlx::Error>;
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
    async fn create(&self, public_key: &str) -> Result<crate::models::wallet::Wallet, sqlx::Error> {
        sqlx::query_as::<_, crate::models::wallet::Wallet>(
            r"
            INSERT INTO wallets (public_key)
            VALUES ($1)
            RETURNING id, public_key, name, created_at, updated_at, deleted_at
            ",
        )
        .bind(public_key)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<crate::models::wallet::Wallet, sqlx::Error> {
        sqlx::query_as::<_, crate::models::wallet::Wallet>(
            r"
            SELECT id, public_key, name, created_at, updated_at, deleted_at
            FROM wallets
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<crate::models::wallet::Wallet>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::wallet::Wallet>(
            r"
            SELECT id, public_key, name, created_at, updated_at, deleted_at
            FROM wallets
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            ",
        )
        .fetch_all(&*self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r"UPDATE wallets SET deleted_at = NOW() WHERE id = $1").bind(id).execute(&*self.pool).await?;
        Ok(())
    }
}
