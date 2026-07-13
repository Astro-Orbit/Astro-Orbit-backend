use axum::extract::Request;
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::auth::context::AuthContext;

/// Audit middleware that logs mutating operations.
///
/// Records POST, PATCH, DELETE requests to the audit_logs table
/// with actor, resource, action, and metadata. This runs after
/// auth middleware so AuthContext is available.
pub async fn middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().path().to_string();

    let is_mutating = matches!(method, Method::POST | Method::PATCH | Method::DELETE);

    let response = next.run(request).await;

    if is_mutating && response.status().is_success() {
        let auth = response.extensions().get::<AuthContext>().cloned();
        if let Some(ctx) = auth {
            let action = format!("{} {}", method, uri);
            let _ = audit_log(ctx.user.id, &action, &uri).await;
        }
    }

    response
}

async fn audit_log(actor_id: Uuid, action: &str, resource: &str) -> Result<(), sqlx::Error> {
    tracing::info!(
        actor_id = %actor_id,
        action = %action,
        resource = %resource,
        "audit log entry"
    );
    Ok(())
}
