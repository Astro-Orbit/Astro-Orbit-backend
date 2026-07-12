use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub public_key: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub id: Uuid,
    pub public_key: String,
    pub name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
