use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::AuthConfig;
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: Uuid,
    pub sid: Uuid,
    pub org: Option<Uuid>,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub aud: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: Uuid,
    pub jti: String,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
}

pub fn issue_access_token(
    user_id: Uuid,
    session_id: Uuid,
    active_org: Option<Uuid>,
    config: &AuthConfig,
    secret: &[u8],
) -> Result<String, AppError> {
    let now = chrono::Utc::now().timestamp();
    let claims = AccessTokenClaims {
        sub: user_id,
        sid: session_id,
        org: active_org,
        iat: now,
        exp: now + i64::try_from(config.access_token_ttl.as_secs()).unwrap_or(i64::MAX),
        iss: config.issuer.clone(),
        aud: config.audience.clone(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
        .map_err(|e| AppError::internal(format!("JWT encode failed: {e}")))
}

pub fn verify_access_token(token: &str, secret: &[u8], config: &AuthConfig) -> Result<AccessTokenClaims, AppError> {
    let mut validation = Validation::default();
    validation.set_issuer(&[&config.issuer]);
    validation.set_audience(&[&config.audience]);
    validation.sub = Some(config.issuer.clone());

    let token_data = decode::<AccessTokenClaims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|_| AppError::Unauthenticated)?;

    Ok(token_data.claims)
}

#[must_use]
pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
