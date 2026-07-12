use async_trait::async_trait;
use uuid::Uuid;

/// Contract service trait.
#[async_trait]
pub trait ContractService: Send + Sync {
    async fn register_contract(
        &self,
        project_id: Uuid,
        name: &str,
        contract_id: &str,
    ) -> Result<ContractResult, crate::AppError>;
    async fn get_contract(&self, contract_id: Uuid) -> Result<ContractResult, crate::AppError>;
    async fn list_contracts(&self, project_id: Uuid) -> Result<Vec<ContractResult>, crate::AppError>;
}

pub struct ContractResult {
    pub id: Uuid,
    pub name: String,
    pub contract_id: String,
    pub verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
