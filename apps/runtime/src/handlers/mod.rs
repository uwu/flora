use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::state::AppState;

pub mod auth;
pub mod deployments;
pub mod error;
pub mod guilds;
pub mod health;
pub mod kv;
pub mod logs;
pub mod metrics;
pub mod response;
pub mod tokens;

/// Build the top-level router with API routes and interactive docs.
pub fn create_router() -> Router<AppState> {
    #[derive(OpenApi)]
    #[openapi(
        nest(
            (path = "/auth", api = auth::AuthApi),
            (path = "/guilds", api = guilds::GuildApi),
            (path = "/tokens", api = tokens::TokenApi),
            (path = "/deployments", api = deployments::DeploymentApi),
            (path = "/kv", api = kv::KvApi),
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

    let api_router = Router::new()
        .nest("/auth", auth::router())
        .nest("/guilds", guilds::router())
        .nest("/tokens", tokens::router())
        .nest("/deployments", deployments::router())
        .nest("/kv", kv::router())
        .route("/health", get(health::health_check))
        .route("/metrics", get(metrics::get_metrics))
        .route("/metrics/json", get(metrics::get_metrics_json))
        .route("/logs", get(logs::get_logs))
        .route("/logs/stream", get(logs::stream_logs))
        .route("/logs/{guild_id}", get(logs::get_guild_logs))
        .route("/logs/{guild_id}/stream", get(logs::stream_guild_logs));

    let oapi_router = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .route("/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    Router::new()
        .nest("/api-docs", oapi_router)
        // Expose API only under `/api/*`
        .nest("/api", api_router)
}
