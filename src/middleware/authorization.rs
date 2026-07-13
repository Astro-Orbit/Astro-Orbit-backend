use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::context::AuthContext;
use crate::permissions::PolicyEngine;
use crate::permissions::role::Role;

/// Authorization middleware that checks a required permission.
///
/// Must be layered AFTER the auth middleware (which inserts AuthContext).
pub async fn require(permission: &'static str) -> impl axum::middleware::Handler<(), _, _> {
    axum::middleware::from_fn(move |request: Request, next: Next| {
        let perm = permission;
        async move {
            let auth = request.extensions().get::<AuthContext>().cloned();

            match auth {
                Some(ctx) => {
                    if ctx.has_permission(perm) {
                        return Ok(next.run(request).await);
                    }
                    Err(forbidden_response(perm))
                }
                None => Err(unauthorized_response()),
            }
        }
    })
}

fn forbidden_response(permission: &str) -> Response {
    let body = serde_json::json!({
        "success": false,
        "error": {
            "code": "FORBIDDEN",
            "message": format!("missing required permission: {permission}"),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    });
    (axum::http::StatusCode::FORBIDDEN, axum::Json(body)).into_response()
}

fn unauthorized_response() -> Response {
    let body = serde_json::json!({
        "success": false,
        "error": {
            "code": "UNAUTHENTICATED",
            "message": "authentication required",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    });
    (axum::http::StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
}
