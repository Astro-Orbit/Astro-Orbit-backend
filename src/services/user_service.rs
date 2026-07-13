use std::sync::Arc;

use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::user_repo::UserRepository;
use crate::repositories::wallet_repo::WalletRepository;

pub struct UserService {
    pool: Arc<sqlx::PgPool>,
    user_repo: Arc<dyn UserRepository>,
    wallet_repo: Arc<dyn WalletRepository>,
}

impl UserService {
    pub fn new(
        pool: Arc<sqlx::PgPool>,
        user_repo: Arc<dyn UserRepository>,
        wallet_repo: Arc<dyn WalletRepository>,
    ) -> Self {
        Self { pool, user_repo, wallet_repo }
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        let user = self.user_repo.find_by_id(user_id).await?;
        let wallets = self.wallet_repo.find_by_user(user_id).await.map_err(AppError::from)?;

        Ok(UserProfile {
            id: user.id,
            stellar_public: user.stellar_public,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            email: user.email,
            email_verified: user.email_verified,
            wallets: wallets
                .iter()
                .map(|w| WalletInfo {
                    id: w.id,
                    public_key: w.public_key.clone(),
                    name: w.name.clone(),
                    is_primary: w.is_primary,
                })
                .collect(),
            created_at: user.created_at,
        })
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
        email: Option<&str>,
    ) -> Result<UserProfile, AppError> {
        let user = self.user_repo.find_by_id(user_id).await?;

        let updated = sqlx::query_as::<_, crate::models::user::User>(
            r"
            UPDATE users
            SET display_name = COALESCE($2, display_name),
                avatar_url = COALESCE($3, avatar_url),
                email = COALESCE($4, email),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, stellar_public, display_name, avatar_url, email, email_verified,
                      totp_enabled, created_at, updated_at, deleted_at
            ",
        )
        .bind(user_id)
        .bind(display_name)
        .bind(avatar_url)
        .bind(email)
        .fetch_one(&*self.pool)
        .await
        .map_err(AppError::from)?;

        let wallets = self.wallet_repo.find_by_user(user_id).await.map_err(AppError::from)?;

        Ok(UserProfile {
            id: user.id,
            stellar_public: user.stellar_public,
            display_name: updated.display_name,
            avatar_url: updated.avatar_url,
            email: updated.email,
            email_verified: updated.email_verified,
            wallets: wallets
                .iter()
                .map(|w| WalletInfo {
                    id: w.id,
                    public_key: w.public_key.clone(),
                    name: w.name.clone(),
                    is_primary: w.is_primary,
                })
                .collect(),
            created_at: updated.created_at,
        })
    }
}

pub struct UserProfile {
    pub id: Uuid,
    pub stellar_public: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub wallets: Vec<WalletInfo>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct WalletInfo {
    pub id: Uuid,
    pub public_key: String,
    pub name: Option<String>,
    pub is_primary: bool,
}
