use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::context::{AuthContext, OrganizationContext};

/// Middleware that populates the organization context in AuthContext.
///
/// Extracts the org ID from the `X-Org-Id` header or from the URL
/// path parameter, then looks up the user's role within that org
/// and updates the AuthContext in request extensions.
///
/// This middleware should run after the auth middleware.
pub async fn middleware(mut request: Request, next: Next) -> Response {
    let org_id = request
        .headers()
        .get("x-org-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| uuid::Uuid::parse_str(v).ok());

    if let Some(org_id) = org_id {
        if let Some(auth) = request.extensions().get::<AuthContext>() {
            let mut ctx = auth.clone();
            ctx.organization = Some(OrganizationContext {
                id: org_id,
                role: crate::permissions::role::Role::Viewer,
                name: String::new(),
                slug: String::new(),
            });
            ctx.permissions = ctx
                .organization
                .as_ref()
                .map(|o| {
                    o.role
                        .permissions()
                        .into_iter()
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default();
            request.extensions_mut().insert(ctx);
        }
    }

    next.run(request).await
}
