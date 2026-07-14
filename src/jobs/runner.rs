use std::sync::Arc;

use crate::events::bus::EventBus;
use crate::jobs::queue::JobQueue;
use crate::jobs::types::Job;

const MAX_RETRIES: i32 = 3;
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

pub struct JobRunner {
    queue: Arc<dyn JobQueue>,
    event_bus: EventBus,
    handlers: Vec<Box<dyn JobHandler>>,
}

#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    fn can_handle(&self, job_type: &str) -> bool;
    async fn handle(&self, job: Job) -> Result<(), String>;
}

impl JobRunner {
    #[must_use]
    pub fn new(queue: Arc<dyn JobQueue>, event_bus: EventBus) -> Self {
        Self { queue, event_bus, handlers: Vec::new() }
    }

    pub fn register(&mut self, handler: Box<dyn JobHandler>) {
        self.handlers.push(handler);
    }

    pub async fn run(&self) {
        tracing::info!("job runner started, polling every {POLL_INTERVAL:?}");

        loop {
            if let Err(e) = self.process_single_job().await {
                tracing::error!("job processing error: {e}");
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    async fn process_single_job(&self) -> Result<(), String> {
        let job = match self.queue.dequeue().await {
            Ok(Some(job)) => job,
            Ok(None) => return Ok(()),
            Err(e) => {
                tracing::error!("failed to dequeue job: {e}");
                return Ok(());
            }
        };

        tracing::info!(job_id = %job.id, job_type = %job.job_type, "processing job");

        let Some(handler) = self.handlers.iter().find(|h| h.can_handle(&job.job_type)) else {
            tracing::warn!(job_id = %job.id, job_type = %job.job_type, "no handler registered");
            self.queue.mark_failed(job.id, "no handler registered").await.map_err(|e| e.to_string())?;
            return Ok(());
        };

        match handler.handle(job.clone()).await {
            Ok(()) => {
                self.queue.mark_completed(job.id).await.map_err(|e| e.to_string())?;
                let _ = self
                    .event_bus
                    .publish(crate::events::types::DomainEvent::new(
                        "job.completed",
                        job.id,
                        "job",
                        serde_json::json!({ "job_type": job.job_type }),
                    ))
                    .await;
            }
            Err(error_msg) => {
                tracing::error!(job_id = %job.id, error = %error_msg, "job failed");

                if job.retry_count < MAX_RETRIES {
                    self.queue.mark_retrying(job.id, &error_msg).await.map_err(|e| e.to_string())?;
                } else {
                    self.queue.send_to_dlq(job.id, &error_msg).await.map_err(|e| e.to_string())?;
                }

                let _ = self
                    .event_bus
                    .publish(crate::events::types::DomainEvent::new(
                        "job.failed",
                        job.id,
                        "job",
                        serde_json::json!({
                            "job_type": job.job_type,
                            "error": error_msg,
                            "retry_count": job.retry_count,
                        }),
                    ))
                    .await;
            }
        }

        Ok(())
    }
}
