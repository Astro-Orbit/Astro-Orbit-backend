use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ContractRepository: Send + Sync {
    async fn create(&self, project_id: Uuid, name: &str, contract_id: &str) -> Result<crate::models::contract::Contract, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<crate::models::contract::Contract, sqlx::Error>;
    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<crate::models::contract::Contract>, sqlx::Error>;
}

pub struct PgContractRepository {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl PgContractRepository {
    #[must_use]
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContractRepository for PgContractRepository {
    async fn create(&self, project_id: Uuid, name: &str, contract_id: &str) -> Result<crate::models::contract::Contract, sqlx::Error> {
        sqlx::query_as::<_, crate::models::contract::Contract>(
            r#"
            INSERT INTO contracts (project_id, name, contract_id)
            VALUES ($1, $2, $3)
            RETURNING id, project_id, name, contract_id, wasm_hash, source_language,
                      abi, verified, created_at, updated_at, deleted_at
            "#,
        )
        .bind(project_id)
        .bind(name)
        .bind(contract_id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<crate::models::contract::Contract, sqlx::Error> {
        sqlx::query_as::<_, crate::models::contract::Contract>(
            r#"
            SELECT id, project_id, name, contract_id, wasm_hash, source_language,
                   abi, verified, created_at, updated_at, deleted_at
            FROM contracts
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_one(&*self.pool)
        .await
    }

    async fn find_by_project(&self, project_id: Uuid) -> Result<Vec<crate::models::contract::Contract>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::contract::Contract>(
            r#"
            SELECT id, project_id, name, contract_id, wasm_hash, source_language,
                   abi, verified, created_at, updated_at, deleted_at
            FROM contracts
            WHERE project_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(&*self.pool)
        .await
    }
}
