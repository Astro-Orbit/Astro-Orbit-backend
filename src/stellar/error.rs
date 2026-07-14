use crate::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum StellarError {
    #[error("RPC call failed: {0}")]
    RpcError(String),

    #[error("contract not found: {0}")]
    ContractNotFound(String),

    #[error("transaction failed: {0}")]
    TransactionFailed(String),

    #[error("invalid account: {0}")]
    InvalidAccount(String),

    #[error("network mismatch: expected {expected}, got {actual}")]
    NetworkMismatch { expected: String, actual: String },

    #[error("rate limited by Soroban RPC")]
    RateLimited,

    #[error("timeout communicating with Soroban RPC")]
    Timeout,

    #[error("serialization error: {0}")]
    SerializationError(String),
}

impl From<StellarError> for AppError {
    fn from(err: StellarError) -> Self {
        match err {
            StellarError::ContractNotFound(msg) => AppError::not_found(msg),
            StellarError::RpcError(_)
            | StellarError::TransactionFailed(_)
            | StellarError::InvalidAccount(_)
            | StellarError::SerializationError(_) => AppError::internal(err.to_string()),
            StellarError::NetworkMismatch { .. } => AppError::bad_request(err.to_string()),
            StellarError::RateLimited => AppError::RateLimited,
            StellarError::Timeout => AppError::unavailable("Soroban RPC timeout"),
        }
    }
}
