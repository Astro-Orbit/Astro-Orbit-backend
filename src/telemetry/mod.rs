use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use crate::config::{Config, LogFormat};

/// Initializes the telemetry stack.
pub fn init(config: &Config) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    let fmt_layer = layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    let fmt_layer = match config.logging.format {
        LogFormat::Json => fmt_layer
            .json()
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(config.logging.enable_ansi)
            .boxed(),
        LogFormat::Text => fmt_layer.pretty().with_target(true).with_ansi(config.logging.enable_ansi).boxed(),
    };

    Registry::default().with(env_filter).with(fmt_layer).init();

    tracing::info!(
        log_format = %config.logging.format,
        log_level = %config.logging.level,
        "telemetry initialized"
    );

    Ok(())
}
