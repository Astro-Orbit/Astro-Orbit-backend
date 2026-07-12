use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::Span;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Middleware that injects a unique request ID into every request.
///
/// The request ID is generated from a timestamp and an atomic counter.
/// It is added to the tracing span and the `X-Request-Id` response header.
pub async fn middleware(request: Request, next: Next) -> Response {
    let request_id = generate_request_id();
    Span::current().record("request_id", &request_id);

    let mut response = next.run(request).await;
    response.headers_mut().insert("X-Request-Id", request_id.parse().unwrap());
    response
}

fn generate_request_id() -> String {
    let count = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
    format!("req_{timestamp:x}_{count:x}")
}
