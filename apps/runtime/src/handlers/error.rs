use std::{collections::BTreeMap, fmt::Display};

use axum::{
    Json,
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;
use utoipa::{ToSchema, openapi::RefOr};
use uuid::Uuid;

use crate::layers::logger::REQUEST_CONTEXT;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProblemViolation {
    /// Identifies the invalid request field using its JSON or parameter path.
    pub field: String,
    /// Explains how to correct the invalid field.
    pub message: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProblemDetails {
    /// Stable relative URI identifying the category of failure.
    #[serde(rename = "type")]
    pub problem_type: String,
    /// Stable, human-readable summary of the failure category.
    pub title: String,
    /// HTTP status code returned for this failure occurrence.
    pub status: u16,
    /// Actionable explanation of this specific failure occurrence.
    pub detail: String,
    /// Request path identifying where this failure occurred.
    pub instance: String,
    /// Correlation identifier also returned in the `X-Request-ID` header.
    pub request_id: String,
    /// Field-level validation failures, when applicable.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<ProblemViolation>,
}

pub type ErrorResponse = ProblemDetails;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    problem_type: &'static str,
    title: &'static str,
    detail: String,
    internal_message: Option<String>,
    violations: Vec<ProblemViolation>,
}

impl ApiError {
    pub fn internal<E: Display>(err: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            problem_type: "/problems/internal-error",
            title: "Internal server error",
            detail: "The server could not complete the request. Try again later and include the request ID when contacting support.".to_string(),
            internal_message: Some(err.to_string()),
            violations: Vec::new(),
        }
    }

    pub fn not_found<M: Into<String>>(detail: M) -> Self {
        Self::client(
            StatusCode::NOT_FOUND,
            "/problems/resource-not-found",
            "Resource not found",
            detail,
        )
    }

    pub fn unauthorized<M: Into<String>>(detail: M) -> Self {
        Self::client(
            StatusCode::UNAUTHORIZED,
            "/problems/authentication-required",
            "Authentication required",
            detail,
        )
    }

    pub fn forbidden<M: Into<String>>(detail: M) -> Self {
        Self::client(
            StatusCode::FORBIDDEN,
            "/problems/access-denied",
            "Access denied",
            detail,
        )
    }

    pub fn bad_request<M: Into<String>>(detail: M) -> Self {
        Self::client(
            StatusCode::BAD_REQUEST,
            "/problems/invalid-request",
            "Invalid request",
            detail,
        )
    }

    pub fn validation<M: Into<String>>(detail: M, violations: Vec<ProblemViolation>) -> Self {
        let mut error = Self::bad_request(detail);
        error.violations = violations;
        error
    }

    pub fn feature_disabled<M: Into<String>>(detail: M) -> Self {
        Self::client(
            StatusCode::FORBIDDEN,
            "/problems/feature-disabled",
            "Feature not enabled",
            detail,
        )
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    fn client<M: Into<String>>(
        status: StatusCode,
        problem_type: &'static str,
        title: &'static str,
        detail: M,
    ) -> Self {
        Self {
            status,
            problem_type,
            title,
            detail: normalize_detail(detail.into()),
            internal_message: None,
            violations: Vec::new(),
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for ApiError {}

impl From<eyre::Report> for ApiError {
    fn from(err: eyre::Report) -> Self {
        Self::internal(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let request_context = REQUEST_CONTEXT.try_with(Clone::clone).ok();
        let (request_id, instance) = match request_context {
            Some(context) => (context.request_id, context.instance),
            None => (Uuid::new_v4(), "/unknown".to_string()),
        };
        let request_id = request_id.to_string();

        if let Some(internal_message) = self.internal_message {
            error!(target: "flora::handlers", request_id, error = internal_message, "internal API error");
        }

        let body = ProblemDetails {
            problem_type: self.problem_type.to_string(),
            title: self.title.to_string(),
            status: self.status.as_u16(),
            detail: self.detail,
            instance,
            request_id,
            violations: self.violations,
        };
        let mut response = (self.status, Json(body)).into_response();
        response.headers_mut().insert(
            CONTENT_TYPE,
            "application/problem+json"
                .parse()
                .expect("valid problem details content type"),
        );
        response
    }
}

impl utoipa::IntoResponses for ApiError {
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        BTreeMap::new()
    }
}

fn normalize_detail(detail: String) -> String {
    let detail = detail.trim();
    if detail.is_empty() {
        return "The request could not be completed.".to_string();
    }

    let mut characters = detail.chars();
    let Some(first) = characters.next() else {
        return "The request could not be completed.".to_string();
    };
    let mut normalized = first.to_uppercase().collect::<String>();
    normalized.push_str(characters.as_str());
    let has_terminal_punctuation = match normalized.chars().last() {
        Some('.') | Some('!') | Some('?') => true,
        Some(_) | None => false,
    };
    if !has_terminal_punctuation {
        normalized.push('.');
    }
    normalized
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

    use super::ApiError;

    #[tokio::test]
    async fn problem_details_are_actionable_and_use_the_standard_media_type() {
        let response = ApiError::bad_request("token is required").into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/problem+json"
        );
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["type"], "/problems/invalid-request");
        assert_eq!(body["title"], "Invalid request");
        assert_eq!(body["detail"], "Token is required.");
        assert!(body["request_id"].as_str().is_some());
    }
}
