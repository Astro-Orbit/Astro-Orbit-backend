use std::sync::Arc;

use axum::extract::Request;
use axum::http::header;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::context::{AuthContext, AuthenticatedUser, SessionInfo};
use crate::auth::jwt::verify_access_token;
use crate::auth::session::{SessionData, SessionStore};
use crate::permissions::role::Role;
use crate::state::AppState;

/// Authentication middleware.
///
/// Validates Bearer JWT, loads Redis session, inserts `AuthContext`.
/// Must be layered BEFORE permission-gated routes.
pub async fn middleware(mut request: Request, next: Next) -> Response {
    let Some(token) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    else {
        return auth_failed("missing authorization header");
    };

    let Some(state) = request.extensions().get::<Arc<RwLock<AppState>>>().cloned() else {
        return auth_failed("application state unavailable");
    };

    let state_guard = state.read().await;
    let config = state_guard.config.clone();

    let Ok(claims) = verify_access_token(token, config.app.secret_key.as_bytes(), &config.auth) else {
        return auth_failed("invalid or expired token");
    };

    let session_data = load_session(claims.sid, &state_guard, &config).await;

    let permissions =
        claims.org.map(|_| Role::Viewer.permissions().into_iter().map(String::from).collect()).unwrap_or_default();

    let expires_at = session_data.as_ref().map_or_else(chrono::Utc::now, |s| s.expires_at);

    let auth_context = AuthContext {
        user: AuthenticatedUser {
            id: claims.sub,
            wallet_address: session_data.as_ref().map(|s| s.public_key.clone()).unwrap_or_default(),
            display_name: None,
        },
        session: SessionInfo { id: claims.sid, expires_at },
        organization: None,
        permissions,
    };

    drop(state_guard);
    request.extensions_mut().insert(auth_context);
    next.run(request).await
}

async fn load_session(session_id: Uuid, state: &AppState, config: &crate::config::Config) -> Option<SessionData> {
    let cache = state.cache.clone()?;
    let mut store = SessionStore::new((*cache).clone(), config.redis.default_ttl);
    store.get(&session_id.to_string()).await.ok().flatten()
}

fn auth_failed(message: &'static str) -> Response {
    let body = serde_json::json!({
        "success": false,
        "error": {
            "code": "UNAUTHENTICATED",
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    });
    (axum::http::StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
}
