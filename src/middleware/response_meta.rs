use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

/// Middleware that enriches responses with timing and version headers.
///
/// Adds `X-Response-Time` and `X-Api-Version` headers to every response.
pub async fn middleware(request: Request, next: Next) -> Response {
    let start = std::time::Instant::now();
    let mut response = next.run(request).await;
    let duration = start.elapsed();

    response.headers_mut().insert(
        "X-Response-Time",
        format!("{}ms", duration.as_millis()).parse().unwrap(),
    );
    response.headers_mut().insert(
        "X-Api-Version",
        env!("CARGO_PKG_VERSION").parse().unwrap(),
    );
    response
}
