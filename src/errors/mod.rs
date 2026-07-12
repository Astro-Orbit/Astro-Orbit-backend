use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::borrow::Cow;

/// Unified application error type.
///
/// Every fallible operation in the system produces an `AppError`.
/// Handlers convert these into structured HTTP error responses
/// that include a request ID and timestamp for traceability.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(Cow<'static, str>),

    #[error("validation error: {0}")]
    Validation(Cow<'static, str>),

    #[error("authentication required")]
    Unauthenticated,

    #[error("forbidden: {0}")]
    Forbidden(Cow<'static, str>),

    #[error("conflict: {0}")]
    Conflict(Cow<'static, str>),

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("internal error: {0}")]
    Internal(Cow<'static, str>),

    #[error("service unavailable: {0}")]
    Unavailable(Cow<'static, str>),

    #[error("{0}")]
    BadRequest(Cow<'static, str>),
}

impl AppError {
    /// Maps the error variant to an HTTP status code.
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Returns a stable machine-readable error code.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Unauthenticated => "UNAUTHENTICATED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Conflict(_) => "CONFLICT",
            Self::RateLimited => "RATE_LIMITED",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Unavailable(_) => "SERVICE_UNAVAILABLE",
            Self::BadRequest(_) => "BAD_REQUEST",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        let body = serde_json::json!({
            "success": false,
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Convenience constructors for common error patterns.
impl AppError {
    #[must_use]
    pub fn not_found(resource: impl Into<Cow<'static, str>>) -> Self {
        Self::NotFound(resource.into())
    }

    #[must_use]
    pub fn validation(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Validation(message.into())
    }

    #[must_use]
    pub fn forbidden(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Forbidden(message.into())
    }

    #[must_use]
    pub fn conflict(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Conflict(message.into())
    }

    #[must_use]
    pub fn internal(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Internal(message.into())
    }

    #[must_use]
    pub fn unavailable(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Unavailable(message.into())
    }

    #[must_use]
    pub fn bad_request(message: impl Into<Cow<'static, str>>) -> Self {
        Self::BadRequest(message.into())
    }
}

/// Domain-level error marker trait.
///
/// Each domain module defines its own error enum and implements this trait
/// to enable conversion into the global `AppError`.
pub trait IntoAppError {
    fn into_app_error(self) -> AppError;
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("resource not found".into()),
            sqlx::Error::Database(ref db_err) => {
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        return Self::Conflict("resource already exists".into());
                    }
                }
                tracing::error!(error = %err, "database error");
                Self::Internal("a database error occurred".into())
            }
            other => {
                tracing::error!(error = %other, "database error");
                Self::Internal("a database error occurred".into())
            }
        }
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!(error = %err, "redis error");
        Self::Unavailable("cache service unavailable".into())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(error = %err, "internal error");
        Self::Internal(err.to_string().into())
    }
}
