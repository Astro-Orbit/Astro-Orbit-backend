use chrono::{DateTime, Utc};

/// Returns the current UTC time.
#[must_use]
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Formats a duration in milliseconds for logging.
#[must_use]
pub fn format_duration_ms(start: std::time::Instant) -> u64 {
    start.elapsed().as_millis() as u64
}
