use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Unified API response envelope.
///
/// Every successful response is wrapped in this struct to provide
/// a consistent contract for API consumers.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
    pub meta: ResponseMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub request_id: String,
    pub timestamp: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates a successful response with the given data.
    #[must_use]
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            pagination: None,
            meta: ResponseMeta {
                request_id: String::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    /// Creates a paginated response.
    #[must_use]
    pub fn paginated(data: T, pagination: PaginationMeta) -> Self {
        Self {
            success: true,
            data,
            pagination: Some(pagination),
            meta: ResponseMeta {
                request_id: String::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    /// Sets the request ID on this response.
    #[must_use]
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.meta.request_id = request_id;
        self
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Empty response for 201 Created endpoints.
#[derive(Debug, Serialize)]
pub struct CreatedResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    pub meta: ResponseMeta,
}

impl<T: Serialize> CreatedResponse<T> {
    #[must_use]
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: ResponseMeta {
                request_id: String::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        }
    }

    #[must_use]
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.meta.request_id = request_id;
        self
    }
}

impl<T: Serialize> IntoResponse for CreatedResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self)).into_response()
    }
}

/// Empty 204 No Content response.
#[derive(Debug, Serialize)]
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}
