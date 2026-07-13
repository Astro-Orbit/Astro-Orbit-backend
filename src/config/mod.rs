use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use serde::Serialize;

/// Application configuration loaded from environment variables.
///
/// All configuration is gathered at startup from environment variables
/// (and `.env` files in development) into an immutable struct. No module
/// should access environment variables directly — everything flows through
/// this struct.
#[derive(Debug, Clone, Serialize)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub stellar: StellarConfig,
    pub telemetry: TelemetryConfig,
    pub rate_limit: RateLimitConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppConfig {
    pub name: String,
    pub env: Environment,
    pub debug: bool,
    pub port: u16,
    pub host: IpAddr,
    pub secret_key: String,
    pub base_url: String,
    pub body_limit: usize,
    pub request_timeout: Duration,
    pub shutdown_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Test,
}

impl Environment {
    #[must_use]
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    #[must_use]
    pub fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }

    #[must_use]
    pub fn is_test(&self) -> bool {
        matches!(self, Self::Test)
    }
}

impl FromStr for Environment {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            "test" | "testing" => Ok(Self::Test),
            other => Err(ConfigError(format!("invalid environment: {other}"))),
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => write!(f, "development"),
            Self::Staging => write!(f, "staging"),
            Self::Production => write!(f, "production"),
            Self::Test => write!(f, "test"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
    pub connect_timeout: Duration,
    pub health_check_interval: Duration,
}

impl DatabaseConfig {
    #[must_use]
    pub fn is_sqlite(&self) -> bool {
        self.url.starts_with("sqlite:")
    }

    #[must_use]
    pub fn is_postgres(&self) -> bool {
        self.url.starts_with("postgres:") || self.url.starts_with("postgresql:")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub default_ttl: Duration,
    pub connect_timeout: Duration,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthConfig {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub challenge_ttl: Duration,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StellarConfig {
    pub rpc_url: String,
    pub network_passphrase: String,
    pub rpc_timeout: Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryConfig {
    pub otlp_endpoint: String,
    pub service_name: String,
    pub metrics_port: u16,
    pub metrics_path: String,
    pub enable_otlp: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub auth_requests_per_minute: u32,
    pub burst_size: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file_path: Option<PathBuf>,
    pub enable_ansi: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LogFormat {
    Json,
    Text,
}

impl LogFormat {
    #[must_use]
    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json)
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Text => write!(f, "text"),
        }
    }
}

impl FromStr for LogFormat {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" | "pretty" => Ok(Self::Text),
            other => Err(ConfigError(format!("invalid log format: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityConfig {
    pub encryption_key: String,
    pub cors_allowed_origins: Vec<String>,
    pub hsts_max_age: Duration,
    pub max_body_size: usize,
}

#[derive(Debug)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "config error: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if any required variable is missing, empty,
    /// or fails to parse. The error message includes the offending key.
    pub fn from_env() -> Result<Self, ConfigError> {
        if !cfg!(feature = "production") {
            dotenvy::dotenv().ok();
        }

        let body_limit: usize = env_parse_opt("APP_BODY_LIMIT_MB").unwrap_or(10) * 1024 * 1024;

        let config = Config {
            app: AppConfig {
                name: env("APP_NAME"),
                env: env_parse("APP_ENV"),
                debug: env_parse("APP_DEBUG"),
                port: env_parse("APP_PORT"),
                host: env_parse("APP_HOST"),
                secret_key: env("APP_SECRET_KEY"),
                base_url: env("APP_BASE_URL"),
                body_limit,
                request_timeout: Duration::from_secs(env_parse_opt("APP_REQUEST_TIMEOUT_SECONDS").unwrap_or(30)),
                shutdown_timeout: Duration::from_secs(env_parse_opt("APP_SHUTDOWN_TIMEOUT_SECONDS").unwrap_or(30)),
            },
            database: DatabaseConfig {
                url: env("DATABASE_URL"),
                max_connections: env_parse("DATABASE_MAX_CONNECTIONS"),
                min_connections: env_parse("DATABASE_MIN_CONNECTIONS"),
                max_lifetime: Duration::from_secs(env_parse::<u64>("DATABASE_MAX_LIFETIME_MINUTES") * 60),
                idle_timeout: Duration::from_secs(env_parse::<u64>("DATABASE_IDLE_TIMEOUT_SECONDS")),
                connect_timeout: Duration::from_secs(env_parse::<u64>("DATABASE_CONNECT_TIMEOUT_SECONDS")),
                health_check_interval: Duration::from_secs(
                    env_parse_opt("DATABASE_HEALTH_CHECK_INTERVAL_SECONDS").unwrap_or(30),
                ),
            },
            redis: RedisConfig {
                url: env("REDIS_URL"),
                pool_size: env_parse("REDIS_POOL_SIZE"),
                default_ttl: Duration::from_secs(env_parse::<u64>("REDIS_DEFAULT_TTL_SECONDS")),
                connect_timeout: Duration::from_secs(env_parse_opt("REDIS_CONNECT_TIMEOUT_SECONDS").unwrap_or(5)),
                retry_attempts: env_parse_opt("REDIS_RETRY_ATTEMPTS").unwrap_or(3),
            },
            auth: AuthConfig {
                access_token_ttl: Duration::from_secs(env_parse::<u64>("JWT_ACCESS_TOKEN_TTL_MINUTES") * 60),
                refresh_token_ttl: Duration::from_secs(env_parse::<u64>("JWT_REFRESH_TOKEN_TTL_DAYS") * 86400),
                challenge_ttl: Duration::from_secs(env_parse_opt("AUTH_CHALLENGE_TTL_SECONDS").unwrap_or(300)),
                issuer: env("JWT_ISSUER"),
                audience: env("JWT_AUDIENCE"),
            },
            stellar: StellarConfig {
                rpc_url: env("STELLAR_RPC_URL"),
                network_passphrase: env("STELLAR_NETWORK_PASSPHRASE"),
                rpc_timeout: Duration::from_secs(env_parse("STELLAR_RPC_TIMEOUT_SECONDS")),
            },
            telemetry: TelemetryConfig {
                otlp_endpoint: env_opt("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_default(),
                service_name: env("OTEL_SERVICE_NAME"),
                metrics_port: env_parse("METRICS_PORT"),
                metrics_path: env_opt("METRICS_PATH").unwrap_or_else(|| String::from("/metrics")),
                enable_otlp: env_parse_opt("OTEL_ENABLED").unwrap_or(false),
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: env_parse("RATE_LIMIT_REQUESTS_PER_MINUTE"),
                auth_requests_per_minute: env_parse("RATE_LIMIT_AUTH_REQUESTS_PER_MINUTE"),
                burst_size: env_parse("RATE_LIMIT_BURST_SIZE"),
                enabled: env_parse_opt("RATE_LIMIT_ENABLED").unwrap_or(true),
            },
            logging: LoggingConfig {
                level: env("LOG_LEVEL"),
                format: env_parse("LOG_FORMAT"),
                file_path: env_opt("LOG_FILE_PATH").map(PathBuf::from),
                enable_ansi: env_parse_opt("LOG_ENABLE_ANSI").unwrap_or(true),
            },
            security: SecurityConfig {
                encryption_key: env("ENCRYPTION_KEY"),
                cors_allowed_origins: env("CORS_ALLOWED_ORIGINS").split(',').map(String::from).collect(),
                hsts_max_age: Duration::from_secs(env_parse_opt("HSTS_MAX_AGE_SECONDS").unwrap_or(31536000)),
                max_body_size: body_limit,
            },
        };

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.app.secret_key.len() < 16 {
            return Err(ConfigError("APP_SECRET_KEY must be at least 16 characters".into()));
        }
        if self.app.name.is_empty() {
            return Err(ConfigError("APP_NAME must not be empty".into()));
        }
        if self.app.base_url.is_empty() {
            return Err(ConfigError("APP_BASE_URL must not be empty".into()));
        }
        if self.database.url.is_empty() {
            return Err(ConfigError("DATABASE_URL must not be empty".into()));
        }
        if self.redis.url.is_empty() {
            return Err(ConfigError("REDIS_URL must not be empty".into()));
        }
        if self.security.encryption_key.len() < 16 {
            return Err(ConfigError("ENCRYPTION_KEY must be at least 16 characters".into()));
        }
        Ok(())
    }
}

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("missing required environment variable: {key}"))
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

fn env_parse<T: FromStr>(key: &str) -> T
where
    T::Err: std::fmt::Display,
{
    std::env::var(key)
        .unwrap_or_else(|_| panic!("missing required environment variable: {key}"))
        .parse()
        .unwrap_or_else(|e| panic!("failed to parse environment variable {key}: {e}"))
}

fn env_parse_opt<T: FromStr>(key: &str) -> Option<T>
where
    T::Err: std::fmt::Display,
{
    std::env::var(key).ok().and_then(|v| {
        if v.is_empty() {
            None
        } else {
            match v.parse() {
                Ok(val) => Some(val),
                Err(e) => panic!("failed to parse environment variable {key}: {e}"),
            }
        }
    })
}

#[cfg(test)]
mod tests;
