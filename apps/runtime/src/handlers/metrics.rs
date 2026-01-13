//! Metrics endpoint for Prometheus scraping.

use axum::{
    http::{StatusCode, header},
    response::IntoResponse,
};
use utoipa::OpenApi;

use crate::metrics;

#[derive(OpenApi)]
#[openapi(paths(get_metrics, get_metrics_json))]
pub struct MetricsApi;

/// Get metrics in Prometheus exposition format.
#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Prometheus metrics", content_type = "text/plain")
    ),
    tag = "metrics"
)]
pub async fn get_metrics() -> impl IntoResponse {
    let body = metrics::metrics().prometheus_format();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
}

/// Get metrics as JSON.
#[utoipa::path(
    get,
    path = "/json",
    responses(
        (status = 200, description = "Metrics as JSON", body = MetricsSnapshot)
    ),
    tag = "metrics"
)]
pub async fn get_metrics_json() -> impl IntoResponse {
    let snapshot = metrics::metrics().snapshot();
    (StatusCode::OK, axum::Json(snapshot))
}

/// Alias for utoipa schema generation.
#[allow(dead_code)]
type MetricsSnapshot = metrics::MetricsSnapshot;
