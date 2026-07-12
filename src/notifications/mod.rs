//! Notification delivery and management.
//!
//! Manages in-app, email, and webhook notification delivery.
//! Subscribes to domain events and routes them to the appropriate
//! delivery channel based on user preferences.
//!
//! Ownership: Notifications Team
//! Dependencies: events, cache, services
//! Public API: `NotificationService`, `NotificationChannel`

pub mod channels;
pub mod preferences;
pub mod template;
