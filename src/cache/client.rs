use std::time::Duration;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::Serialize;
use tracing::instrument;

use crate::config::RedisConfig;
use crate::errors::AppError;

/// High-level Redis cache client built on `redis::ConnectionManager`.
///
/// Provides typed get/set/delete operations with configurable TTL
/// and automatic connection management.
#[derive(Clone)]
pub struct CacheClient {
    conn: ConnectionManager,
    default_ttl: Duration,
}

impl CacheClient {
    /// Creates a new cache client by connecting to Redis.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` if the connection cannot be established.
    pub async fn connect(config: &RedisConfig) -> Result<Self, AppError> {
        let client = redis::Client::open(config.url.as_str())
            .map_err(|e| AppError::unavailable(format!("invalid Redis URL: {e}")))?;

        let conn = ConnectionManager::new(client).await.map_err(|e| {
            tracing::error!(error = %e, "failed to connect to Redis");
            AppError::unavailable(format!("Redis connection failed: {e}"))
        })?;

        tracing::info!("cache client connected to Redis");

        Ok(Self { conn, default_ttl: config.default_ttl })
    }

    /// Returns `true` if the underlying connection is still alive.
    pub async fn check_health(&mut self) -> Result<Duration, AppError> {
        let start = std::time::Instant::now();
        redis::cmd("PING")
            .query_async::<String>(&mut self.conn)
            .await
            .map_err(|e| AppError::unavailable(format!("Redis health check failed: {e}")))?;
        Ok(start.elapsed())
    }

    /// Gets a value from the cache.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    /// Returns `Ok(None)` if the key does not exist.
    #[instrument(skip(self), fields(key = %key))]
    pub async fn get<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, AppError> {
        let value: Option<String> = self.conn.get(key).await.map_err(|e| {
            tracing::error!(error = %e, key = %key, "cache get failed");
            AppError::unavailable(format!("cache get failed: {e}"))
        })?;

        match value {
            Some(raw) => serde_json::from_str(&raw)
                .map(Some)
                .map_err(|e| AppError::internal(format!("cache deserialization failed: {e}"))),
            None => Ok(None),
        }
    }

    /// Sets a value in the cache with the default TTL.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self, value), fields(key = %key))]
    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), AppError> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Sets a value in the cache with a specific TTL.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self, value), fields(key = %key, ttl_secs = ttl.as_secs()))]
    pub async fn set_with_ttl<T: Serialize>(&mut self, key: &str, value: &T, ttl: Duration) -> Result<(), AppError> {
        let raw =
            serde_json::to_string(value).map_err(|e| AppError::internal(format!("cache serialization failed: {e}")))?;

        let _: () = self.conn.set_ex(key, raw, ttl.as_secs()).await.map_err(|e| {
            tracing::error!(error = %e, key = %key, "cache set failed");
            AppError::unavailable(format!("cache set failed: {e}"))
        })?;

        Ok(())
    }

    /// Deletes a key from the cache.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self), fields(key = %key))]
    pub async fn delete(&mut self, key: &str) -> Result<bool, AppError> {
        let deleted: i32 = self.conn.del(key).await.map_err(|e| {
            tracing::error!(error = %e, key = %key, "cache delete failed");
            AppError::unavailable(format!("cache delete failed: {e}"))
        })?;
        Ok(deleted > 0)
    }

    /// Checks if a key exists in the cache.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self), fields(key = %key))]
    pub async fn exists(&mut self, key: &str) -> Result<bool, AppError> {
        let exists: bool = self.conn.exists(key).await.map_err(|e| {
            tracing::error!(error = %e, key = %key, "cache exists failed");
            AppError::unavailable(format!("cache exists failed: {e}"))
        })?;
        Ok(exists)
    }

    /// Returns the remaining TTL of a key in seconds.
    /// Returns `None` if the key does not exist or has no expiry.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self), fields(key = %key))]
    pub async fn ttl(&mut self, key: &str) -> Result<Option<Duration>, AppError> {
        let ttl: i64 = self.conn.ttl(key).await.map_err(|e| {
            tracing::error!(error = %e, key = %key, "cache ttl failed");
            AppError::unavailable(format!("cache ttl failed: {e}"))
        })?;
        if ttl < 0 {
            Ok(None)
        } else {
            Ok(Some(Duration::from_secs(ttl as u64)))
        }
    }

    /// Increments a numeric value at the given key.
    /// Creates the key with value 0 if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self), fields(key = %key))]
    pub async fn increment(&mut self, key: &str, amount: i64) -> Result<i64, AppError> {
        let result: i64 = self.conn.incr(key, amount).await.map_err(|e| {
            tracing::error!(error = %e, key = %key, "cache increment failed");
            AppError::unavailable(format!("cache increment failed: {e}"))
        })?;
        Ok(result)
    }

    /// Sets a TTL on an existing key.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self), fields(key = %key, ttl_secs = ttl.as_secs()))]
    pub async fn expire(&mut self, key: &str, ttl: Duration) -> Result<bool, AppError> {
        let result: bool =
            self.conn.expire(key, i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX)).await.map_err(|e| {
                tracing::error!(error = %e, key = %key, "cache expire failed");
                AppError::unavailable(format!("cache expire failed: {e}"))
            })?;
        Ok(result)
    }
}
