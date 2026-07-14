use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::broadcast;

use crate::events::types::DomainEvent;

type EventHandler = Arc<dyn Fn(DomainEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct EventBus {
    tx: broadcast::Sender<DomainEvent>,
    handlers: Arc<tokio::sync::RwLock<Vec<EventHandler>>>,
}

impl EventBus {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx, handlers: Arc::new(tokio::sync::RwLock::new(Vec::new())) }
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.tx.subscribe()
    }

    pub async fn publish(&self, event: DomainEvent) -> Result<(), crate::AppError> {
        let handlers = self.handlers.read().await;

        if let Err(e) = self.tx.send(event.clone()) {
            tracing::warn!(event_type = %event.event_type, "event broadcast failed: {e}");
        }

        for handler in handlers.iter() {
            let ev = event.clone();
            let fut = (handler)(ev);
            tokio::spawn(async move {
                fut.await;
            });
        }

        Ok(())
    }

    pub async fn register_handler(&self, handler: EventHandler) {
        self.handlers.write().await.push(handler);
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self { tx: self.tx.clone(), handlers: Arc::clone(&self.handlers) }
    }
}
