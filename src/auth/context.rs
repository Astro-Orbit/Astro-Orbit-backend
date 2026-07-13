use std::collections::HashSet;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use uuid::Uuid;

use crate::errors::AppError;
use crate::permissions::role::Role;

/// Authenticated user summary loaded into the request context.
///
/// Pre-loaded by the auth middleware so handlers rarely need
/// additional database lookups.
#[derive(Debug, Clone, Serialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub wallet_address: String,
    pub display_name: Option<String>,
}

/// Session metadata attached to the current request.
#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Organization context when acting within one.
#[derive(Debug, Clone, Serialize)]
pub struct OrganizationContext {
    pub id: Uuid,
    pub role: Role,
    pub name: String,
    pub slug: String,
}

/// Rich authentication context injected into every authenticated request.
///
/// Extracted via `auth: AuthContext` in handler signatures.
/// All fields are pre-populated by the auth middleware chain.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: AuthenticatedUser,
    pub session: SessionInfo,
    pub organization: Option<OrganizationContext>,
    pub permissions: HashSet<String>,
}

impl AuthContext {
    /// Returns the user's role within the active org, or `None`.
    #[must_use]
    pub fn role(&self) -> Option<&Role> {
        self.organization.as_ref().map(|o| &o.role)
    }

    /// Returns true if the user has the specified permission.
    #[must_use]
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Convenience: returns the active org ID or an error.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` if no org is active.
    pub fn require_org(&self) -> Result<Uuid, AppError> {
        self.organization.as_ref().map(|o| o.id).ok_or_else(|| AppError::forbidden("no active organization"))
    }
}

/// Custom extractor that retrieves `AuthContext` from request extensions.
///
/// The auth middleware (`auth::middleware`) must have run before this
/// extractor is used, otherwise it returns a 401 Unauthorized response.
impl<S: Send + Sync> FromRequestParts<S> for AuthContext {
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = parts.extensions.get::<AuthContext>().cloned().ok_or_else(|| {
            let body = serde_json::json!({
                "success": false,
                "error": {
                    "code": "UNAUTHENTICATED",
                    "message": "authentication required",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }
            });
            (axum::http::StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
        });
        async move { result }
    }
}
