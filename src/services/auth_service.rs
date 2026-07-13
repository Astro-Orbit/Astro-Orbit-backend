use std::sync::Arc;

use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::challenge::generate_challenge;
use crate::auth::jwt::{hash_refresh_token, issue_access_token};
use crate::auth::session::{SessionData, SessionStore};
use crate::auth::wallet::verify_stellar_signature;
use crate::cache::client::CacheClient;
use crate::config::Config;
use crate::errors::AppError;
use crate::repositories::api_key_repo::ApiKeyRepository;
use crate::repositories::challenge_repo::ChallengeRepository;
use crate::repositories::user_repo::UserRepository;
use crate::repositories::wallet_repo::WalletRepository;
use crate::utils::crypto::random_hex;

pub struct AuthService {
    pool: Arc<sqlx::PgPool>,
    cache: Arc<CacheClient>,
    config: Arc<Config>,
    user_repo: Arc<dyn UserRepository>,
    wallet_repo: Arc<dyn WalletRepository>,
    challenge_repo: Arc<dyn ChallengeRepository>,
}

impl AuthService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: Arc<sqlx::PgPool>,
        cache: Arc<CacheClient>,
        config: Arc<Config>,
        user_repo: Arc<dyn UserRepository>,
        wallet_repo: Arc<dyn WalletRepository>,
        challenge_repo: Arc<dyn ChallengeRepository>,
    ) -> Self {
        Self { pool, cache, config, user_repo, wallet_repo, challenge_repo }
    }

    pub async fn generate_challenge(&self, public_key: &str) -> Result<(String, Uuid), AppError> {
        let challenge = generate_challenge();
        let expires_at = Utc::now() + self.config.auth.challenge_ttl;

        let record = self.challenge_repo.create(public_key, &challenge, expires_at).await?;

        Ok((challenge, record.id))
    }

    pub async fn verify_and_login(
        &self,
        public_key: &str,
        signature: &str,
        challenge_id: Uuid,
    ) -> Result<AuthResult, AppError> {
        let record = self.challenge_repo.find_by_id(challenge_id).await?;

        crate::auth::challenge::validate_challenge(&record.expires_at, &record.used_at)?;

        verify_stellar_signature(public_key, &record.challenge, signature)?;

        self.challenge_repo.mark_used(challenge_id).await?;

        let user = match self.user_repo.find_by_stellar_public(public_key).await {
            Ok(u) => u,
            Err(_) => self.user_repo.create(public_key).await.map_err(AppError::from)?,
        };

        let _wallet = self.wallet_repo.find_or_create(public_key, user.id).await?;

        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let access_token =
            issue_access_token(user.id, session_id, None, &self.config.auth, self.config.app.secret_key.as_bytes())?;

        let refresh_token = random_hex::<32>();
        let refresh_token_hash = hash_refresh_token(&refresh_token);

        let session_expires = now + self.config.auth.refresh_token_ttl;
        let session_data = SessionData {
            user_id: user.id,
            public_key: public_key.to_string(),
            created_at: now,
            expires_at: session_expires,
            device_type: None,
            metadata: serde_json::json!({}),
        };

        let mut store = SessionStore::new((*self.cache).clone(), self.config.redis.default_ttl);
        store.store(&session_id.to_string(), &session_data).await?;

        sqlx::query(
            r"
            INSERT INTO sessions (id, user_id, refresh_token_hash, expires_at)
            VALUES ($1, $2, $3, $4)
            ",
        )
        .bind(session_id)
        .bind(user.id)
        .bind(&refresh_token_hash)
        .bind(session_expires)
        .execute(&*self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(AuthResult {
            access_token,
            refresh_token,
            user_id: user.id,
            public_key: public_key.to_string(),
            display_name: user.display_name,
        })
    }

    pub async fn refresh_session(&self, refresh_token: &str) -> Result<AuthResult, AppError> {
        let token_hash = hash_refresh_token(refresh_token);

        let session_row = sqlx::query_as::<_, (Uuid, Uuid, chrono::DateTime<Utc>)>(
            r"
            SELECT id, user_id, expires_at
            FROM sessions
            WHERE refresh_token_hash = $1 AND revoked_at IS NULL
            ",
        )
        .bind(&token_hash)
        .fetch_optional(&*self.pool)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::Unauthenticated)?;

        let (session_id, user_id, expires_at) = session_row;

        if Utc::now() > expires_at {
            return Err(AppError::Unauthenticated);
        }

        let user = self.user_repo.find_by_id(user_id).await?;

        let new_refresh_token = random_hex::<32>();
        let new_refresh_hash = hash_refresh_token(&new_refresh_token);
        let new_expires = Utc::now() + self.config.auth.refresh_token_ttl;

        sqlx::query(
            r"
            UPDATE sessions
            SET refresh_token_hash = $1, expires_at = $2
            WHERE id = $3
            ",
        )
        .bind(&new_refresh_hash)
        .bind(new_expires)
        .bind(session_id)
        .execute(&*self.pool)
        .await
        .map_err(AppError::from)?;

        let access_token =
            issue_access_token(user_id, session_id, None, &self.config.auth, self.config.app.secret_key.as_bytes())?;

        let mut store = SessionStore::new((*self.cache).clone(), self.config.redis.default_ttl);
        if let Ok(Some(mut data)) = store.get(&session_id.to_string()).await {
            data.expires_at = new_expires;
            store.store(&session_id.to_string(), &data).await.ok();
        }

        Ok(AuthResult {
            access_token,
            refresh_token: new_refresh_token,
            user_id: user.id,
            public_key: user.stellar_public,
            display_name: user.display_name,
        })
    }

    pub async fn logout(&self, session_id: Uuid) -> Result<(), AppError> {
        let mut store = SessionStore::new((*self.cache).clone(), self.config.redis.default_ttl);
        store.delete(&session_id.to_string()).await?;

        sqlx::query(
            r"
            UPDATE sessions SET revoked_at = NOW() WHERE id = $1
            ",
        )
        .bind(session_id)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn logout_all(&self, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query(
            r"
            UPDATE sessions SET revoked_at = NOW()
            WHERE user_id = $1 AND revoked_at IS NULL
            ",
        )
        .bind(user_id)
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn verify_api_key(&self, key: &str) -> Result<Uuid, AppError> {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let key_hash = hex::encode(hasher.finalize());

        let repo = crate::repositories::api_key_repo::PgApiKeyRepository::new(self.pool.clone());
        let api_key = repo.find_by_hash(&key_hash).await?;

        if let Some(expires) = api_key.expires_at {
            if Utc::now() > expires {
                return Err(AppError::bad_request("API key expired"));
            }
        }

        repo.update_last_used(api_key.id).await?;

        Ok(api_key.created_by)
    }
}

pub struct AuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub public_key: String,
    pub display_name: Option<String>,
}
