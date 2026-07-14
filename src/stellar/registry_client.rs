use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::stellar::client::SorobanClient;
use crate::stellar::error::StellarError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryContract {
    pub contract_id: String,
    pub name: String,
    pub wasm_hash: String,
    pub version: String,
    pub owner: String,
    pub deployer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub contract_id: String,
    pub name: String,
    pub wasm_hash: String,
    pub version: String,
    pub metadata: Value,
}

/// Client for interacting with on-chain Soroban contract registries.
///
/// A registry is a Soroban contract that maintains a directory of
/// deployed contracts, their versions, and metadata. This client
/// provides methods to query and register contracts on-chain.
pub struct RegistryClient<'a> {
    client: &'a SorobanClient,
    registry_contract_id: String,
}

impl<'a> RegistryClient<'a> {
    #[must_use]
    pub fn new(client: &'a SorobanClient, registry_contract_id: String) -> Self {
        Self { client, registry_contract_id }
    }

    pub async fn lookup_contract(&self, name: &str) -> Result<Option<RegistryEntry>, StellarError> {
        let key = format!("contract:{name}");
        let result = self.client.get_contract_data(&self.registry_contract_id, &key).await;

        match result {
            Ok(data) => {
                let entry: RegistryEntry =
                    serde_json::from_value(data).map_err(|e| StellarError::SerializationError(e.to_string()))?;
                Ok(Some(entry))
            }
            Err(StellarError::ContractNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_contracts(&self) -> Result<Vec<RegistryEntry>, StellarError> {
        let key = "all_contracts".to_string();
        let result = self.client.get_contract_data(&self.registry_contract_id, &key).await?;

        let entries: Vec<RegistryEntry> =
            serde_json::from_value(result).map_err(|e| StellarError::SerializationError(e.to_string()))?;
        Ok(entries)
    }

    pub async fn get_contract_by_hash(&self, wasm_hash: &str) -> Result<Option<RegistryEntry>, StellarError> {
        let key = format!("wasm:{wasm_hash}");
        let result = self.client.get_contract_data(&self.registry_contract_id, &key).await;

        match result {
            Ok(data) => {
                let entry: RegistryEntry =
                    serde_json::from_value(data).map_err(|e| StellarError::SerializationError(e.to_string()))?;
                Ok(Some(entry))
            }
            Err(StellarError::ContractNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
