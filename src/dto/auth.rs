use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ChallengeRequest {
    #[validate(length(min = 32, max = 64))]
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyRequest {
    #[validate(length(min = 32, max = 64))]
    pub public_key: String,
    #[validate(length(min = 1))]
    pub signature: String,
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}
