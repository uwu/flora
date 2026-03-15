use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request},
    routing::get,
};
#[cfg(not(debug_assertions))]
use axum::{
    http::{HeaderValue, header},
    middleware::{self, Next},
    response::Response,
};
use tower_http::compression::CompressionLayer;
#[cfg(not(debug_assertions))]
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::state::AppState;

pub mod auth;
pub mod builds;
pub mod deployments;
pub mod error;
pub mod guilds;
pub mod health;
pub mod kv;
pub mod logs;
pub mod metrics;
pub mod response;
pub mod secrets;
pub mod tokens;

#[cfg(not(debug_assertions))]
const ASSET_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";
#[cfg(not(debug_assertions))]
const FRONTEND_CACHE_CONTROL: &str = "public, max-age=3600";
#[cfg(not(debug_assertions))]
const HTML_CACHE_CONTROL: &str = "no-cache";

#[cfg(not(debug_assertions))]
async fn set_asset_cache_headers(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(ASSET_CACHE_CONTROL),
    );
    response
}

#[cfg(not(debug_assertions))]
async fn set_frontend_cache_headers(req: Request, next: Next) -> Response {
    let path = req.uri().path();
    let is_html = path == "/" || path.ends_with(".html") || !path.contains('.');
    let is_service_worker = path == "/sw.js";
    let mut response = next.run(req).await;
    let header_value = if is_html || is_service_worker {
        HTML_CACHE_CONTROL
    } else {
        FRONTEND_CACHE_CONTROL
    };
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(header_value),
    );
    response
}

/// Build the top-level router with API routes and interactive docs.
pub fn create_router() -> Router<AppState> {
    #[derive(OpenApi)]
    #[openapi(
        nest(
            (path = "/auth", api = auth::AuthApi),
            (path = "/builds", api = builds::BuildApi),
            (path = "/guilds", api = guilds::GuildApi),
            (path = "/tokens", api = tokens::TokenApi),
            (path = "/deployments", api = deployments::DeploymentApi),
            (path = "/kv", api = kv::KvApi),
            (path = "/secrets", api = secrets::SecretsApi),
            (path = "/health", api = health::HealthApi),
            (path = "/metrics", api = metrics::MetricsApi),
            (path = "/logs", api = logs::LogsApi)
        ),
        tags(
            (name = "flora", description = "Flora bot runtime API")
        ),
        // Advertise that the actual HTTP base is /api so generated clients hit the right URLs.
        servers((url = "/api", description = "API base path"))
    )]
    struct ApiDoc;

    let compressed_api = Router::new()
        .nest("/auth", auth::router())
        .nest("/builds", builds::router())
        .nest("/guilds", guilds::router())
        .nest("/tokens", tokens::router())
        .nest("/deployments", deployments::router())
        .nest("/secrets", secrets::router())
        .nest("/kv", kv::router())
        .route("/health", get(health::health_check))
        .route("/metrics", get(metrics::get_metrics))
        .route("/metrics/json", get(metrics::get_metrics_json))
        .layer(CompressionLayer::new());

    let logs_router = Router::new()
        .route("/logs", get(logs::get_logs))
        .route("/logs/stream", get(logs::stream_logs))
        .route("/logs/{guild_id}", get(logs::get_guild_logs))
        .route("/logs/{guild_id}/stream", get(logs::stream_guild_logs));

    let api_router = Router::new().merge(compressed_api).merge(logs_router);

    let oapi_router = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .route("/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    let router = Router::new()
        .nest("/api-docs", oapi_router)
        // Expose API only under `/api/*`
        .nest("/api", api_router)
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024));

    #[cfg(not(debug_assertions))]
    {
        let assets_router = Router::new()
            .fallback_service(ServeDir::new("apps/frontend/dist/assets"))
            .layer(middleware::from_fn(set_asset_cache_headers));

        let frontend_router = Router::new()
            .fallback_service(
                ServeDir::new("apps/frontend/dist")
                    .fallback(ServeFile::new("apps/frontend/dist/index.html")),
            )
            .layer(middleware::from_fn(set_frontend_cache_headers));

        return router
            .nest("/assets", assets_router)
            .fallback_service(frontend_router);
    }

    #[cfg(debug_assertions)]
    {
        router
    }
}
