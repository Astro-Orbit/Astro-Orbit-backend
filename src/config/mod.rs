use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

/// Application configuration loaded from environment variables.
///
/// All configuration is gathered at startup from environment variables
/// (and `.env` files in development) into an immutable struct. No module
/// should access environment variables directly — everything flows through
/// this struct.
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub name: String,
    pub env: Environment,
    pub debug: bool,
    pub port: u16,
    pub host: IpAddr,
    pub secret_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Test,
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            "test" => Ok(Self::Test),
            other => Err(format!("invalid environment: {other}")),
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

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: Duration,
    pub idle_timeout: Duration,
    pub connect_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub default_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone)]
pub struct StellarConfig {
    pub rpc_url: String,
    pub network_passphrase: String,
    pub rpc_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub otlp_endpoint: String,
    pub service_name: String,
    pub metrics_port: u16,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub auth_requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogFormat {
    Json,
    Text,
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" | "pretty" => Ok(Self::Text),
            other => Err(format!("invalid log format: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub encryption_key: String,
    pub cors_allowed_origins: Vec<String>,
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
    pub fn from_env() -> Result<Self, ConfigError> {
        if cfg!(not(feature = "production")) {
            dotenvy::dotenv().ok();
        }

        Ok(Self {
            app: AppConfig {
                name: env("APP_NAME"),
                env: env_parse("APP_ENV"),
                debug: env_parse("APP_DEBUG"),
                port: env_parse("APP_PORT"),
                host: env_parse("APP_HOST"),
                secret_key: env("APP_SECRET_KEY"),
                base_url: env("APP_BASE_URL"),
            },
            database: DatabaseConfig {
                url: env("DATABASE_URL"),
                max_connections: env_parse("DATABASE_MAX_CONNECTIONS"),
                min_connections: env_parse("DATABASE_MIN_CONNECTIONS"),
                max_lifetime: Duration::from_secs(env_parse::<u64>("DATABASE_MAX_LIFETIME_MINUTES") * 60),
                idle_timeout: Duration::from_secs(env_parse::<u64>("DATABASE_IDLE_TIMEOUT_SECONDS")),
                connect_timeout: Duration::from_secs(env_parse::<u64>("DATABASE_CONNECT_TIMEOUT_SECONDS")),
            },
            redis: RedisConfig {
                url: env("REDIS_URL"),
                pool_size: env_parse("REDIS_POOL_SIZE"),
                default_ttl: Duration::from_secs(env_parse::<u64>("REDIS_DEFAULT_TTL_SECONDS")),
            },
            auth: AuthConfig {
                access_token_ttl: Duration::from_secs(env_parse::<u64>("JWT_ACCESS_TOKEN_TTL_MINUTES") * 60),
                refresh_token_ttl: Duration::from_secs(env_parse::<u64>("JWT_REFRESH_TOKEN_TTL_DAYS") * 86400),
                issuer: env("JWT_ISSUER"),
                audience: env("JWT_AUDIENCE"),
            },
            stellar: StellarConfig {
                rpc_url: env("STELLAR_RPC_URL"),
                network_passphrase: env("STELLAR_NETWORK_PASSPHRASE"),
                rpc_timeout: Duration::from_secs(env_parse("STELLAR_RPC_TIMEOUT_SECONDS")),
            },
            telemetry: TelemetryConfig {
                otlp_endpoint: env("OTEL_EXPORTER_OTLP_ENDPOINT"),
                service_name: env("OTEL_SERVICE_NAME"),
                metrics_port: env_parse("METRICS_PORT"),
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: env_parse("RATE_LIMIT_REQUESTS_PER_MINUTE"),
                auth_requests_per_minute: env_parse("RATE_LIMIT_AUTH_REQUESTS_PER_MINUTE"),
                burst_size: env_parse("RATE_LIMIT_BURST_SIZE"),
            },
            logging: LoggingConfig {
                level: env("LOG_LEVEL"),
                format: env_parse("LOG_FORMAT"),
                file_path: env_optional("LOG_FILE_PATH"),
            },
            security: SecurityConfig {
                encryption_key: env("ENCRYPTION_KEY"),
                cors_allowed_origins: env("CORS_ALLOWED_ORIGINS")
                    .split(',')
                    .map(String::from)
                    .collect(),
            },
        })
    }
}

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("missing required environment variable: {key}"))
}

fn env_optional(key: &str) -> Option<String> {
    std::env::var(key).ok()
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
