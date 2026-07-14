use std::time::Duration;

use reqwest::Client as HttpClient;
use serde_json::Value;

use crate::config::StellarConfig;
use crate::stellar::error::StellarError;

pub struct SorobanClient {
    http: HttpClient,
    rpc_url: String,
    network_passphrase: String,
    _timeout: Duration,
}

impl SorobanClient {
    #[must_use]
    pub fn new(config: &StellarConfig) -> Self {
        let http = HttpClient::builder()
            .timeout(config.rpc_timeout)
            .build()
            .expect("failed to build HTTP client for Soroban RPC");

        Self {
            http,
            rpc_url: config.rpc_url.clone(),
            network_passphrase: config.network_passphrase.clone(),
            _timeout: config.rpc_timeout,
        }
    }

    fn json_rpc_request(method: &str, params: &Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        })
    }

    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value, StellarError> {
        let body = Self::json_rpc_request(method, &params);

        let response = self.http.post(&self.rpc_url).json(&body).send().await.map_err(|e| {
            if e.is_timeout() {
                StellarError::Timeout
            } else {
                StellarError::RpcError(e.to_string())
            }
        })?;

        let json: Value = response.json().await.map_err(|e| StellarError::RpcError(e.to_string()))?;

        if let Some(error) = json.get("error") {
            let code = error.get("code").and_then(Value::as_i64).unwrap_or(-1);
            let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
            if code == -32005 {
                return Err(StellarError::RateLimited);
            }
            return Err(StellarError::RpcError(format!("code {code}: {message}")));
        }

        json.get("result").cloned().ok_or_else(|| StellarError::RpcError("empty RPC response".into()))
    }

    pub async fn get_contract_data(&self, contract_id: &str, key: &str) -> Result<Value, StellarError> {
        let params = serde_json::json!({
            "contractId": contract_id,
            "key": key,
        });
        self.rpc_call("getContractData", params).await
    }

    pub async fn get_transaction_status(&self, hash: &str) -> Result<Value, StellarError> {
        let params = serde_json::json!({
            "hash": hash,
        });
        self.rpc_call("getTransaction", params).await
    }

    pub async fn get_ledger_entries(&self, keys: Vec<Value>) -> Result<Value, StellarError> {
        let params = serde_json::json!({
            "keys": keys,
        });
        self.rpc_call("getLedgerEntries", params).await
    }

    pub async fn get_network(&self) -> Result<Value, StellarError> {
        self.rpc_call("getNetwork", serde_json::json!({})).await
    }

    pub async fn get_account(&self, address: &str) -> Result<Value, StellarError> {
        let params = serde_json::json!({
            "address": address,
        });
        self.rpc_call("getAccount", params).await
    }

    #[must_use]
    pub fn network_passphrase(&self) -> &str {
        &self.network_passphrase
    }
}
