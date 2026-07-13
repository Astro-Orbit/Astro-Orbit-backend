pub mod auth;
mod request_id;
mod response_meta;

use axum::http::HeaderName;
use axum::Router;
use tower_http::cors::{Any, CorsLayer, ExposeHeaders};
use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::timeout::RequestBodyTimeoutLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::config::Config;

/// Applies all global middleware to the router.
///
/// Middleware is applied bottom-to-top (last added executes first):
/// 1. Sensitive headers masking (authorization, cookies)
/// 2. Request ID generation and propagation
/// 3. HTTP tracing
/// 4. CORS
/// 5. Request body timeout
/// 6. Response metadata (headers)
pub fn apply<S: Clone + Send + Sync + 'static>(config: &Config, router: Router<S>) -> Router<S> {
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .security
                .cors_allowed_origins
                .iter()
                .map(|origin| origin.parse().expect("invalid CORS origin"))
                .collect::<Vec<_>>(),
        )
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(ExposeHeaders::any());

    router
        .layer(axum::middleware::from_fn(response_meta::middleware))
        .layer(axum::middleware::from_fn(request_id::middleware))
        .layer(RequestBodyTimeoutLayer::new(config.app.request_timeout))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(HeaderName::from_static("x-request-id")))
        .layer(SetRequestIdLayer::new(HeaderName::from_static("x-request-id"), UuidRequestIdMaker))
        .layer(SetSensitiveRequestHeadersLayer::new([
            axum::http::header::AUTHORIZATION,
            axum::http::header::COOKIE,
            axum::http::header::SET_COOKIE,
        ]))
}

/// Generates UUID-based request IDs.
#[derive(Clone, Copy)]
struct UuidRequestIdMaker;

impl MakeRequestId for UuidRequestIdMaker {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        axum::http::HeaderValue::from_str(&id).ok().map(RequestId::new)
    }
}
