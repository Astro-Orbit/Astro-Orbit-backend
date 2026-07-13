use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cache::client::CacheClient;
use crate::errors::AppError;

/// Session data stored in Redis for fast auth checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: Uuid,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub device_type: Option<String>,
    pub metadata: serde_json::Value,
}

/// Redis-backed session cache.
pub struct SessionStore {
    client: CacheClient,
    prefix: String,
    default_ttl: Duration,
}

impl SessionStore {
    const KEY_PREFIX: &'static str = "session";

    #[must_use]
    pub fn new(client: CacheClient, default_ttl: Duration) -> Self {
        Self { client, prefix: Self::KEY_PREFIX.to_string(), default_ttl }
    }

    fn session_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.prefix, session_id)
    }

    pub async fn store(&mut self, session_id: &str, data: &SessionData) -> Result<(), AppError> {
        let key = self.session_key(session_id);
        self.client.set_with_ttl(&key, data, self.default_ttl).await
    }

    pub async fn get(&mut self, session_id: &str) -> Result<Option<SessionData>, AppError> {
        let key = self.session_key(session_id);
        self.client.get(&key).await
    }

    pub async fn delete(&mut self, session_id: &str) -> Result<bool, AppError> {
        let key = self.session_key(session_id);
        self.client.delete(&key).await
    }

    pub async fn touch(&mut self, session_id: &str) -> Result<bool, AppError> {
        let key = self.session_key(session_id);
        if self.client.exists(&key).await? {
            self.client.expire(&key, self.default_ttl).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
