//! Metrics endpoint for Prometheus scraping.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
};
use utoipa::OpenApi;

use crate::{
    handlers::{
        auth::require_identity,
        error::ApiError,
        response::{ApiJson, ApiText},
    },
    metrics,
    state::AppState,
};

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
pub async fn get_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiText, ApiError> {
    require_identity(&state, &headers).await?;
    let body = metrics::metrics().prometheus_format();
    Ok(ApiText((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )))
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
pub async fn get_metrics_json(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<MetricsSnapshot>, ApiError> {
    require_identity(&state, &headers).await?;
    let snapshot = metrics::metrics().snapshot();
    Ok(ApiJson(axum::Json(snapshot)))
}

/// Alias for utoipa schema generation.
#[allow(dead_code)]
type MetricsSnapshot = metrics::MetricsSnapshot;
