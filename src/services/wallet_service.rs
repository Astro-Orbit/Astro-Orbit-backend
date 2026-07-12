use async_trait::async_trait;
use uuid::Uuid;

/// Wallet service trait.
#[async_trait]
pub trait WalletService: Send + Sync {
    async fn register_wallet(&self, public_key: &str) -> Result<WalletResult, crate::AppError>;
    async fn get_wallet(&self, wallet_id: Uuid) -> Result<WalletResult, crate::AppError>;
    async fn list_wallets(&self) -> Result<Vec<WalletResult>, crate::AppError>;
    async fn delete_wallet(&self, wallet_id: Uuid) -> Result<(), crate::AppError>;
}

pub struct WalletResult {
    pub id: Uuid,
    pub public_key: String,
    pub name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
