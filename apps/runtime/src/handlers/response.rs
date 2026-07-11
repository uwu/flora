use std::collections::BTreeMap;

use axum::{
    Json, http,
    response::{IntoResponse, Response},
};
use cookie::Cookie;
use serde::Serialize;
use utoipa::{IntoResponses, openapi::RefOr};

/// JSON wrapper that also carries utoipa response metadata.
#[derive(Debug)]
pub struct ApiJson<T>(pub Json<T>);

impl<T> IntoResponse for ApiJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

impl<T> From<Json<T>> for ApiJson<T> {
    fn from(value: Json<T>) -> Self {
        Self(value)
    }
}

impl<T> IntoResponses for ApiJson<T>
where
    T: Serialize,
{
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        BTreeMap::new()
    }
}

/// Plain text wrapper for endpoints that return text bodies.
#[derive(Debug)]
pub struct ApiText(
    pub  (
        http::StatusCode,
        [(http::header::HeaderName, &'static str); 1],
        String,
    ),
);

impl IntoResponse for ApiText {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl IntoResponses for ApiText {
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        BTreeMap::new()
    }
}

/// Simple redirect wrapper that documents a 302.
pub struct ApiRedirect {
    pub response: Response,
}

impl IntoResponse for ApiRedirect {
    fn into_response(self) -> Response {
        self.response
    }
}

impl IntoResponses for ApiRedirect {
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        BTreeMap::new()
    }
}

/// Redirect response with attached Set-Cookie headers.
pub struct ApiRedirectWithCookies {
    pub response: Response,
    pub cookies: Vec<Cookie<'static>>,
}

/// Response wrapper for Server-Sent Event streams.
pub struct ApiEventStream(pub Response);

impl IntoResponse for ApiEventStream {
    fn into_response(self) -> Response {
        self.0
    }
}

impl IntoResponses for ApiEventStream {
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        BTreeMap::new()
    }
}

impl IntoResponse for ApiRedirectWithCookies {
    fn into_response(self) -> Response {
        let mut response = self.response;
        for cookie in self.cookies {
            if let Ok(value) = http::HeaderValue::from_str(&cookie.to_string()) {
                response
                    .headers_mut()
                    .append(http::header::SET_COOKIE, value);
            }
        }
        response
    }
}

impl IntoResponses for ApiRedirectWithCookies {
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        BTreeMap::new()
    }
}
