use std::sync::Arc;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::api_key_repo::ApiKeyRepository;

pub struct ApiKeyService {
    api_key_repo: Arc<dyn ApiKeyRepository>,
}

impl ApiKeyService {
    pub fn new(_pool: Arc<sqlx::PgPool>, api_key_repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { api_key_repo }
    }

    pub async fn create(
        &self,
        org_id: Option<Uuid>,
        user_id: Uuid,
        name: &str,
        scopes: &[String],
        expires_in_days: Option<i64>,
    ) -> Result<CreatedApiKey, AppError> {
        let raw_key = format!("ao_{}", crate::utils::crypto::random_hex::<32>());
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let key_hash = hex::encode(hasher.finalize());
        let key_prefix = raw_key[..10].to_string();

        let expires_at = expires_in_days.map(|days| chrono::Utc::now() + chrono::Duration::days(days));

        let api_key =
            self.api_key_repo.create(org_id, user_id, name, &key_hash, &key_prefix, scopes, expires_at).await?;

        Ok(CreatedApiKey {
            id: api_key.id,
            name: api_key.name,
            key: raw_key,
            prefix: key_prefix,
            scopes: scopes.to_vec(),
            expires_at,
            created_at: api_key.created_at,
        })
    }

    pub async fn list(&self, org_id: Option<Uuid>, user_id: Uuid) -> Result<Vec<ApiKeyInfo>, AppError> {
        let keys = if let Some(oid) = org_id {
            self.api_key_repo.find_by_org(oid).await?
        } else {
            self.api_key_repo.find_by_user(user_id).await?
        };

        Ok(keys
            .iter()
            .map(|k| {
                let scopes: Vec<String> = serde_json::from_value(k.permissions.clone()).unwrap_or_default();
                ApiKeyInfo {
                    id: k.id,
                    name: k.name.clone(),
                    prefix: k.key_prefix.clone(),
                    scopes,
                    last_used_at: k.last_used_at,
                    expires_at: k.expires_at,
                    created_at: k.created_at,
                }
            })
            .collect())
    }

    pub async fn delete(&self, key_id: Uuid) -> Result<(), AppError> {
        self.api_key_repo.delete(key_id).await
    }
}

pub struct CreatedApiKey {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct ApiKeyInfo {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
