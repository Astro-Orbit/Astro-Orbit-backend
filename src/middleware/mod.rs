mod request_id;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;

/// Applies all global middleware to the router.
pub fn apply(config: &Config, router: Router) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .security
                .cors_allowed_origins
                .iter()
                .map(|origin| origin.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    router
        .layer(axum::middleware::from_fn(request_id::middleware))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
