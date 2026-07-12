//! Internal event bus for domain events.
//!
//! Provides an asynchronous event dispatch mechanism using
//! `tokio::sync::broadcast` for in-process subscribers and
//! `PostgreSQL` LISTEN/NOTIFY for cross-process delivery.
//!
//! Ownership: Infrastructure Team
//! Dependencies: models, config
//! Public API: `EventBus`, Event, `EventHandler`, publish, subscribe

pub mod bus;
pub mod types;
