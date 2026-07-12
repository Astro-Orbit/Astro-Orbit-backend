use async_trait::async_trait;

/// Authentication service trait.
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn generate_challenge(&self, public_key: &str) -> Result<ChallengeResult, crate::AppError>;
    async fn verify_signature(&self, request: VerifyRequest) -> Result<AuthResult, crate::AppError>;
    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthResult, crate::AppError>;
    async fn revoke_session(&self, session_id: &str) -> Result<(), crate::AppError>;
}

pub struct ChallengeResult {
    pub challenge: String,
    pub session_id: String,
}

pub struct VerifyRequest {
    pub public_key: String,
    pub signature: String,
    pub session_id: String,
}

pub struct AuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user: serde_json::Value,
}
