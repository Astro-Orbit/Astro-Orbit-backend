use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tracing::instrument;

use crate::errors::AppError;

/// Sliding window rate limiter backed by Redis.
///
/// Uses a sorted set with timestamps as scores to track request windows.
pub struct RateLimiter {
    conn: ConnectionManager,
    prefix: String,
}

impl RateLimiter {
    const KEY_PREFIX: &'static str = "ratelimit";

    #[must_use]
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn, prefix: Self::KEY_PREFIX.to_string() }
    }

    fn window_key(&self, namespace: &str, key: &str) -> String {
        format!("{}:{}:{}", self.prefix, namespace, key)
    }

    /// Checks if a request should be allowed.
    ///
    /// Returns `Ok(true)` if the request is within limits,
    /// `Ok(false)` if rate limited.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Unavailable` on Redis errors.
    #[instrument(skip(self), fields(namespace = %namespace, key = %key, max_requests = max_requests, window_secs = window_secs))]
    pub async fn check(
        &mut self,
        namespace: &str,
        key: &str,
        max_requests: u32,
        window_secs: u32,
    ) -> Result<bool, AppError> {
        let window_key = self.window_key(namespace, key);
        let now: f64 = chrono::Utc::now().timestamp_millis() as f64;
        let window_start = (now - f64::from(window_secs) * 1000.0) as i64;

        let script = redis::Script::new(
            r"redis.call('ZREMRANGEBYSCORE', KEYS[1], '-inf', ARGV[1])
            local count = redis.call('ZCARD', KEYS[1])
            if count < tonumber(ARGV[2]) then
                redis.call('ZADD', KEYS[1], ARGV[3], ARGV[3])
                redis.call('EXPIRE', KEYS[1], ARGV[4])
                return 1
            else
                return 0
            end",
        );

        let allowed: bool = script
            .key(window_key)
            .arg(window_start)
            .arg(max_requests)
            .arg(now as i64)
            .arg(window_secs)
            .invoke_async(&mut self.conn)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "rate limiter check failed");
                AppError::unavailable(format!("rate limiter check failed: {e}"))
            })?;

        Ok(allowed)
    }

    /// Returns the current count of requests in the window for a key.
    #[instrument(skip(self), fields(namespace = %namespace, key = %key))]
    pub async fn count(&mut self, namespace: &str, key: &str) -> Result<u32, AppError> {
        let window_key = self.window_key(namespace, key);
        let count: u32 = self.conn.zcard(&window_key).await.map_err(|e| {
            tracing::error!(error = %e, "rate limiter count failed");
            AppError::unavailable(format!("rate limiter count failed: {e}"))
        })?;
        Ok(count)
    }

    /// Resets the rate limit window for a given key.
    #[instrument(skip(self), fields(namespace = %namespace, key = %key))]
    pub async fn reset(&mut self, namespace: &str, key: &str) -> Result<(), AppError> {
        let window_key = self.window_key(namespace, key);
        let _: i32 = self.conn.del(&window_key).await.map_err(|e| {
            tracing::error!(error = %e, "rate limiter reset failed");
            AppError::unavailable(format!("rate limiter reset failed: {e}"))
        })?;
        Ok(())
    }
}
