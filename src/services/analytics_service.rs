use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AnalyticsService: Send + Sync {
    async fn org_stats(&self, org_id: Uuid) -> Result<OrgStats, crate::AppError>;
    async fn user_activity_feed(&self, user_id: Uuid, limit: i64) -> Result<Vec<ActivityEntry>, crate::AppError>;
}

pub struct OrgStats {
    pub total_projects: i64,
    pub total_contracts: i64,
    pub total_deployments: i64,
    pub active_deployments: i64,
    pub total_members: i64,
    pub activities_24h: i64,
}

pub struct ActivityEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub actor_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct AnalyticsServiceImpl {
    org: std::sync::Arc<dyn crate::repositories::org_repo::OrgRepository>,
    project: std::sync::Arc<dyn crate::repositories::project_repo::ProjectRepository>,
    contract: std::sync::Arc<dyn crate::repositories::contract_repo::ContractRepository>,
    deployment: std::sync::Arc<dyn crate::repositories::deployment_repo::DeploymentRepository>,
    activity: std::sync::Arc<dyn crate::repositories::activity_repo::ActivityRepository>,
}

impl AnalyticsServiceImpl {
    #[must_use]
    pub fn new(
        org: std::sync::Arc<dyn crate::repositories::org_repo::OrgRepository>,
        project: std::sync::Arc<dyn crate::repositories::project_repo::ProjectRepository>,
        contract: std::sync::Arc<dyn crate::repositories::contract_repo::ContractRepository>,
        deployment: std::sync::Arc<dyn crate::repositories::deployment_repo::DeploymentRepository>,
        activity: std::sync::Arc<dyn crate::repositories::activity_repo::ActivityRepository>,
    ) -> Self {
        Self { org, project, contract, deployment, activity }
    }
}

#[async_trait]
impl AnalyticsService for AnalyticsServiceImpl {
    async fn org_stats(&self, org_id: Uuid) -> Result<OrgStats, crate::AppError> {
        let projects = self.project.find_by_org(org_id).await?;
        let total_projects = i64::try_from(projects.len()).unwrap_or(i64::MAX);

        let mut total_contracts: i64 = 0;
        let mut total_deployments: i64 = 0;
        let mut active_deployments: i64 = 0;

        for project in &projects {
            let contracts = self.contract.find_by_project(project.id).await?;
            total_contracts += i64::try_from(contracts.len()).unwrap_or(i64::MAX);

            let deployments = self.deployment.find_by_project(project.id).await?;
            total_deployments += i64::try_from(deployments.len()).unwrap_or(i64::MAX);
            active_deployments +=
                i64::try_from(deployments.iter().filter(|d| d.status == "deployed" || d.status == "building").count())
                    .unwrap_or(i64::MAX);
        }

        let total_members = self.org.member_count(org_id).await?;
        let since = chrono::Utc::now() - chrono::Duration::hours(24);
        let activities_24h = self.activity.count_by_org_since(org_id, since).await?;

        Ok(OrgStats {
            total_projects,
            total_contracts,
            total_deployments,
            active_deployments,
            total_members,
            activities_24h,
        })
    }

    async fn user_activity_feed(&self, user_id: Uuid, limit: i64) -> Result<Vec<ActivityEntry>, crate::AppError> {
        let activities = self.activity.find_by_user(user_id, limit).await?;
        Ok(activities
            .into_iter()
            .map(|a| ActivityEntry {
                id: a.id,
                organization_id: a.organization_id,
                actor_id: a.actor_id,
                action: a.action,
                resource_type: a.resource_type,
                resource_id: a.resource_id,
                metadata: a.metadata,
                created_at: a.created_at,
            })
            .collect())
    }
}
