use std::sync::Arc;

use uuid::Uuid;

use crate::errors::AppError;
use crate::models::wallet::Wallet;
use crate::repositories::wallet_repo::WalletRepository;

pub struct WalletService {
    wallet_repo: Arc<dyn WalletRepository>,
}

impl WalletService {
    pub fn new(wallet_repo: Arc<dyn WalletRepository>) -> Self {
        Self { wallet_repo }
    }

    pub async fn link_wallet(&self, public_key: &str, user_id: Uuid) -> Result<Wallet, AppError> {
        self.wallet_repo.find_or_create(public_key, user_id).await
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Wallet>, AppError> {
        self.wallet_repo.find_by_user(user_id).await.map_err(AppError::from)
    }

    pub async fn set_primary(&self, wallet_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.wallet_repo.set_primary(wallet_id, user_id).await.map_err(AppError::from)
    }
}
