use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::state::AppState;

pub mod auth;
pub mod deployments;
pub mod error;
pub mod guilds;
pub mod health;
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
            (path = "/health", api = health::HealthApi)
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
        .route("/health", get(health::health_check));

    let oapi_router = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .route("/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    Router::new()
        .nest("/api-docs", oapi_router)
        // Expose API only under `/api/*`
        .nest("/api", api_router)
}
