use std::collections::BTreeMap;

use axum::{
    Json, http,
    response::{IntoResponse, Response},
};
use cookie::Cookie;
use serde::Serialize;
use utoipa::{
    IntoResponses, ToSchema,
    openapi::{RefOr, content::ContentBuilder, response::ResponseBuilder},
};

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
    T: ToSchema + Serialize,
{
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        let content = ContentBuilder::new().schema(Some(T::schema())).build();
        let response = ResponseBuilder::new()
            .description("Successful response")
            .content("application/json", content)
            .build();

        BTreeMap::from([("200".to_string(), RefOr::T(response))])
    }
}

/// JSON response with attached Set-Cookie headers.
pub struct ApiJsonWithCookies<T> {
    pub payload: ApiJson<T>,
    pub cookies: Vec<Cookie<'static>>,
}

impl<T> IntoResponse for ApiJsonWithCookies<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut response = self.payload.into_response();
        for cookie in self.cookies {
            if let Ok(value) = http::HeaderValue::from_str(&cookie.to_string()) {
                response.headers_mut().append(http::header::SET_COOKIE, value);
            }
        }
        response
    }
}

impl<T> IntoResponses for ApiJsonWithCookies<T>
where
    T: ToSchema + Serialize,
{
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        ApiJson::<T>::responses()
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
        let response = ResponseBuilder::new().description("Redirect response").build();
        BTreeMap::from([("302".to_string(), RefOr::T(response))])
    }
}

/// Redirect response with attached Set-Cookie headers.
pub struct ApiRedirectWithCookies {
    pub response: Response,
    pub cookies: Vec<Cookie<'static>>,
}

impl IntoResponse for ApiRedirectWithCookies {
    fn into_response(self) -> Response {
        let mut response = self.response;
        for cookie in self.cookies {
            if let Ok(value) = http::HeaderValue::from_str(&cookie.to_string()) {
                response.headers_mut().append(http::header::SET_COOKIE, value);
            }
        }
        response
    }
}

impl IntoResponses for ApiRedirectWithCookies {
    fn responses() -> BTreeMap<String, RefOr<utoipa::openapi::response::Response>> {
        let response = ResponseBuilder::new().description("Redirect response with cookies").build();
        BTreeMap::from([("302".to_string(), RefOr::T(response))])
    }
}
