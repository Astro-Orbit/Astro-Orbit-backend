//! In-process event bus implementation.
//!
//! Uses `tokio::sync::broadcast` with configurable channel capacity.
//! Each event type is dispatched to all registered handlers.
