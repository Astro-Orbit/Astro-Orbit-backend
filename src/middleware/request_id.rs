use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Span;

/// Middleware that records the request ID (set by `SetRequestIdLayer`)
/// into the active tracing span for structured log correlation.
pub async fn middleware(request: Request, next: Next) -> Response {
    let request_id =
        request.headers().get("x-request-id").and_then(|v| v.to_str().ok()).unwrap_or("unknown").to_string();

    Span::current().record("request_id", &request_id);
    next.run(request).await
}
