use super::*;
use serial_test::serial;

/// Sets an environment variable for the duration of a test, restoring
/// the original value when the guard is dropped.
struct EnvGuard {
    key: String,
    previous: Option<String>,
}

impl EnvGuard {
    fn set(key: &str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key: key.to_string(), previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(v) => std::env::set_var(&self.key, v),
            None => std::env::remove_var(&self.key),
        }
    }
}

fn set_required_envs() -> Vec<EnvGuard> {
    vec![
        EnvGuard::set("APP_NAME", "test-app"),
        EnvGuard::set("APP_ENV", "development"),
        EnvGuard::set("APP_DEBUG", "true"),
        EnvGuard::set("APP_PORT", "8080"),
        EnvGuard::set("APP_HOST", "0.0.0.0"),
        EnvGuard::set("APP_SECRET_KEY", "test-secret-key-16ch"),
        EnvGuard::set("APP_BASE_URL", "http://localhost:8080"),
        EnvGuard::set("DATABASE_URL", "postgres://localhost:5432/test"),
        EnvGuard::set("DATABASE_MAX_CONNECTIONS", "10"),
        EnvGuard::set("DATABASE_MIN_CONNECTIONS", "2"),
        EnvGuard::set("DATABASE_MAX_LIFETIME_MINUTES", "30"),
        EnvGuard::set("DATABASE_IDLE_TIMEOUT_SECONDS", "300"),
        EnvGuard::set("DATABASE_CONNECT_TIMEOUT_SECONDS", "10"),
        EnvGuard::set("REDIS_URL", "redis://localhost:6379"),
        EnvGuard::set("REDIS_POOL_SIZE", "5"),
        EnvGuard::set("REDIS_DEFAULT_TTL_SECONDS", "300"),
        EnvGuard::set("JWT_ACCESS_TOKEN_TTL_MINUTES", "15"),
        EnvGuard::set("JWT_REFRESH_TOKEN_TTL_DAYS", "30"),
        EnvGuard::set("JWT_ISSUER", "test"),
        EnvGuard::set("JWT_AUDIENCE", "test"),
        EnvGuard::set("STELLAR_RPC_URL", "https://testnet.stellar.org"),
        EnvGuard::set("STELLAR_NETWORK_PASSPHRASE", "Test SDF Network ; September 2015"),
        EnvGuard::set("STELLAR_RPC_TIMEOUT_SECONDS", "30"),
        EnvGuard::set("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318"),
        EnvGuard::set("OTEL_SERVICE_NAME", "test"),
        EnvGuard::set("METRICS_PORT", "9090"),
        EnvGuard::set("RATE_LIMIT_REQUESTS_PER_MINUTE", "100"),
        EnvGuard::set("RATE_LIMIT_AUTH_REQUESTS_PER_MINUTE", "10"),
        EnvGuard::set("RATE_LIMIT_BURST_SIZE", "20"),
        EnvGuard::set("LOG_LEVEL", "info"),
        EnvGuard::set("LOG_FORMAT", "json"),
        EnvGuard::set("ENCRYPTION_KEY", "encryption-key-16c"),
        EnvGuard::set("CORS_ALLOWED_ORIGINS", "http://localhost:3000"),
    ]
}

#[test]
#[serial]
fn test_config_from_env_success() {
    let _guards = set_required_envs();
    let config = Config::from_env().expect("config should load");
    assert_eq!(config.app.name, "test-app");
    assert_eq!(config.app.env, Environment::Development);
    assert!(config.app.debug);
    assert_eq!(config.app.port, 8080);
}

#[test]
#[serial]
fn test_config_environment_parsing() {
    let _guards = set_required_envs();

    let env = "production".parse::<Environment>().unwrap();
    assert_eq!(env, Environment::Production);
    assert!(env.is_production());

    let env = "dev".parse::<Environment>().unwrap();
    assert_eq!(env, Environment::Development);
    assert!(env.is_development());

    let env = "test".parse::<Environment>().unwrap();
    assert_eq!(env, Environment::Test);
    assert!(env.is_test());
}

#[test]
#[serial]
fn test_config_validation_fails_on_short_secret_key() {
    let mut guards = set_required_envs();
    guards.push(EnvGuard::set("APP_SECRET_KEY", "short"));
    let result = Config::from_env();
    assert!(result.is_err());
    assert!(result.unwrap_err().0.contains("APP_SECRET_KEY"));
}

#[test]
#[serial]
fn test_config_validation_fails_on_empty_base_url() {
    let mut guards = set_required_envs();
    guards.push(EnvGuard::set("APP_BASE_URL", ""));
    let result = Config::from_env();
    assert!(result.is_err());
    assert!(result.unwrap_err().0.contains("APP_BASE_URL"));
}

#[test]
#[serial]
fn test_log_format_parsing() {
    assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
    assert_eq!("text".parse::<LogFormat>().unwrap(), LogFormat::Text);
    assert_eq!("pretty".parse::<LogFormat>().unwrap(), LogFormat::Text);
    assert!("invalid".parse::<LogFormat>().is_err());
}

#[test]
#[serial]
fn test_environment_helpers() {
    let prod = Environment::Production;
    assert!(prod.is_production());
    assert!(!prod.is_development());
    assert!(!prod.is_test());

    let dev = Environment::Development;
    assert!(dev.is_development());
    assert!(!dev.is_production());
}

#[test]
#[serial]
fn test_database_config_helpers() {
    let mut guards = set_required_envs();
    guards.push(EnvGuard::set("DATABASE_URL", "postgres://localhost:5432/test"));
    let config = Config::from_env().unwrap();
    assert!(config.database.is_postgres());
    assert!(!config.database.is_sqlite());
}
