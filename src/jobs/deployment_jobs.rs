use std::sync::Arc;

use uuid::Uuid;

use crate::deployment::pipeline::{run_build_stage, run_deploy_stage, run_scan_stage, PipelineContext, StageResult};
use crate::jobs::runner::JobHandler;
use crate::jobs::types::Job;
use crate::stellar::client::SorobanClient;

pub struct DeploymentJobHandler {
    stellar: Option<Arc<SorobanClient>>,
}

impl DeploymentJobHandler {
    #[must_use]
    pub fn new(stellar: Option<Arc<SorobanClient>>) -> Self {
        Self { stellar }
    }
}

#[async_trait::async_trait]
impl JobHandler for DeploymentJobHandler {
    fn can_handle(&self, job_type: &str) -> bool {
        matches!(job_type, "deploy.build" | "deploy.scan" | "deploy.deploy" | "deploy.rollback")
    }

    async fn handle(&self, job: Job) -> Result<(), String> {
        let deployment_id: Uuid = job.payload["deployment_id"]
            .as_str()
            .ok_or_else(|| "missing deployment_id".to_string())?
            .parse()
            .map_err(|e: uuid::Error| e.to_string())?;

        let project_id: Uuid = job.payload["project_id"]
            .as_str()
            .ok_or_else(|| "missing project_id".to_string())?
            .parse()
            .map_err(|e: uuid::Error| e.to_string())?;

        let environment =
            job.payload["environment"].as_str().ok_or_else(|| "missing environment".to_string())?.to_string();

        let contract_ids: Vec<Uuid> = job.payload["contract_ids"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).filter_map(|s| s.parse().ok()).collect())
            .unwrap_or_default();

        let ctx =
            PipelineContext { deployment_id, project_id, environment, contract_ids, stellar: self.stellar.clone() };

        let result = match job.job_type.as_str() {
            "deploy.build" => run_build_stage(&ctx).await,
            "deploy.scan" => run_scan_stage(&ctx).await,
            "deploy.deploy" => run_deploy_stage(&ctx).await,
            "deploy.rollback" => {
                tracing::info!(deployment_id = %deployment_id, "rollback requested");
                return Ok(());
            }
            _ => return Err("unknown deployment job type".to_string()),
        };

        match result {
            StageResult::Success(_state) => Ok(()),
            StageResult::Failure(msg) => Err(msg),
        }
    }
}
