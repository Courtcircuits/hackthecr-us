use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, ToSchema, Error)]
pub enum ApiError {
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Unprocessable Entity: {0}")]
    #[allow(dead_code)]
    UnProcessableEntity(String),
    #[error("Not Found: {0}")]
    #[allow(dead_code)]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    #[allow(dead_code)]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    #[allow(dead_code)]
    Forbidden(String),
    #[error("Bad Request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),
    #[error("Service Unavailable: {0}")]
    #[allow(dead_code)]
    ServiceUnavailable(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({
            "error": self.to_string(),
        });

        let status_code = match self {
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::UnProcessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };

        (status_code, axum::Json(body)).into_response()

    }
}
