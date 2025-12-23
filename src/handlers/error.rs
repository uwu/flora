use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;
use utoipa::openapi::{RefOr, content::ContentBuilder, response::ResponseBuilder};
use utoipa::{PartialSchema, ToSchema};

/// Canonical error envelope returned by the HTTP API.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Human readable error message.
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The requested resource does not exist.
    #[error("resource not found: {message}")]
    NotFound { message: String },
    /// Authentication is required or invalid.
    #[error("unauthorized: {message}")]
    Unauthorized { message: String },
    /// The request was understood but refused.
    #[error("forbidden: {message}")]
    Forbidden { message: String },
    /// Client sent invalid input.
    #[error("bad request: {message}")]
    BadRequest { message: String },
    /// Any unrecoverable server error.
    #[error("internal server error")]
    Internal { message: String },
}

impl ApiError {
    pub fn internal<E: std::fmt::Display>(err: E) -> Self {
        ApiError::Internal { message: err.to_string() }
    }

    pub fn not_found<M: Into<String>>(message: M) -> Self {
        ApiError::NotFound { message: message.into() }
    }

    pub fn unauthorized<M: Into<String>>(message: M) -> Self {
        ApiError::Unauthorized { message: message.into() }
    }

    pub fn forbidden<M: Into<String>>(message: M) -> Self {
        ApiError::Forbidden { message: message.into() }
    }

    pub fn bad_request<M: Into<String>>(message: M) -> Self {
        ApiError::BadRequest { message: message.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::Internal { ref message } => {
                error!(target: "flora::handlers", "Internal API error: {}", message);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = ErrorResponse { message: self.to_string() };
        (status, Json(body)).into_response()
    }
}

impl utoipa::IntoResponses for ApiError {
    fn responses() -> std::collections::BTreeMap<String, RefOr<utoipa::openapi::response::Response>>
    {
        let content = ContentBuilder::new().schema(Some(ErrorResponse::schema())).build();
        let not_found = ResponseBuilder::new()
            .description("Resource not found")
            .content("application/json", content.clone())
            .build();
        let unauthorized = ResponseBuilder::new()
            .description("Authentication required")
            .content("application/json", content.clone())
            .build();
        let forbidden = ResponseBuilder::new()
            .description("Forbidden")
            .content("application/json", content.clone())
            .build();
        let bad_request = ResponseBuilder::new()
            .description("Bad request")
            .content("application/json", content.clone())
            .build();
        let internal = ResponseBuilder::new()
            .description("Internal server error")
            .content("application/json", content)
            .build();

        std::collections::BTreeMap::from([
            ("404".to_string(), RefOr::T(not_found)),
            ("401".to_string(), RefOr::T(unauthorized)),
            ("403".to_string(), RefOr::T(forbidden)),
            ("400".to_string(), RefOr::T(bad_request)),
            ("500".to_string(), RefOr::T(internal)),
        ])
    }
}
