use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub event_type: String,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub payload: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent {
    #[must_use]
    pub fn new(event_type: &str, aggregate_id: Uuid, aggregate_type: &str, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            aggregate_id,
            aggregate_type: aggregate_type.to_string(),
            payload,
            occurred_at: Utc::now(),
        }
    }
}
