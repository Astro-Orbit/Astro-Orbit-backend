use crate::config::{Config, LogFormat};
use std::str::FromStr;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

/// Initializes the telemetry stack.
pub fn init(config: &Config) {
    let level_filter = EnvFilter::from_str(&config.logging.level).unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer: Box<dyn Layer<_> + Send + Sync> = match config.logging.format {
        LogFormat::Json => {
            tracing_subscriber::fmt::layer().json().with_target(true).with_file(true).with_line_number(true).boxed()
        }
        LogFormat::Text => tracing_subscriber::fmt::layer().pretty().with_target(true).boxed(),
    };

    let layered = fmt_layer.and_then(level_filter);
    let registry = tracing_subscriber::registry().with(layered);

    registry.init();

    tracing::info!(
        log_format = %config.logging.format,
        log_level = %config.logging.level,
        "telemetry initialized"
    );
}
