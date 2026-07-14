use crate::jobs::runner::JobHandler;
use crate::jobs::types::Job;

pub struct NotificationJobHandler;

#[async_trait::async_trait]
impl JobHandler for NotificationJobHandler {
    fn can_handle(&self, job_type: &str) -> bool {
        matches!(job_type, "notification.email" | "notification.webhook" | "notification.in_app")
    }

    async fn handle(&self, job: Job) -> Result<(), String> {
        let recipient = job.payload["recipient"].as_str().unwrap_or("unknown");
        let title = job.payload["title"].as_str().unwrap_or("");

        match job.job_type.as_str() {
            "notification.email" => {
                tracing::info!(recipient, title, "sending email notification");
            }
            "notification.webhook" => {
                let url = job.payload["webhook_url"].as_str().unwrap_or("");
                tracing::info!(url, "sending webhook notification");
            }
            "notification.in_app" => {
                tracing::info!(recipient, title, "sending in-app notification");
            }
            _ => return Err("unknown notification job type".to_string()),
        }

        Ok(())
    }
}
