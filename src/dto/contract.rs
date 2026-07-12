use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub name: String,
    pub contract_id: String,
    pub wasm_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContractResponse {
    pub id: Uuid,
    pub name: String,
    pub contract_id: String,
    pub wasm_hash: Option<String>,
    pub verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVersionRequest {
    pub wasm_hash: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ContractVersionResponse {
    pub id: Uuid,
    pub version: String,
    pub wasm_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
