use axum::Json;
use serde::{Deserialize, Serialize};

use crate::response::{ApiResponse, CreatedResponse, NoContent};

#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub public_key: String,
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

pub async fn challenge(Json(_req): Json<ChallengeRequest>) -> CreatedResponse<ChallengeResponse> {
    CreatedResponse::new(ChallengeResponse {
        challenge: String::new(),
        session_id: String::new(),
    })
}

pub async fn verify(Json(_req): Json<VerifyRequest>) -> CreatedResponse<VerifyResponse> {
    CreatedResponse::new(VerifyResponse {
        access_token: String::new(),
        refresh_token: String::new(),
        user: serde_json::Value::Null,
    })
}

pub async fn refresh(Json(_req): Json<RefreshRequest>) -> ApiResponse<RefreshResponse> {
    ApiResponse::success(RefreshResponse {
        access_token: String::new(),
        refresh_token: String::new(),
    })
}

pub async fn logout() -> NoContent {
    NoContent
}

pub async fn session() -> ApiResponse<serde_json::Value> {
    ApiResponse::success(serde_json::json!({}))
}
