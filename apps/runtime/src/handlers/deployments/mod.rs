use axum::{Router, routing::get};
use utoipa::OpenApi;

use crate::state::AppState;

pub mod list;
pub mod read;
pub mod upsert;

pub use list::list_deployments_handler;
pub use read::get_deployment_handler;
pub use upsert::{DeploymentRequest, DeploymentResponse, upsert_deployment_handler};

/// Deployment API surface.
#[derive(OpenApi)]
#[openapi(
    paths(
        list::list_deployments_handler,
        read::get_deployment_handler,
        upsert::upsert_deployment_handler
    ),
    components(schemas(DeploymentRequest, DeploymentResponse, super::error::ErrorResponse)),
    tags((name = "deployment", description = "Manage per-guild bot deployments"))
)]
pub struct DeploymentApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_deployments_handler))
        .route(
            "/{guild_id}",
            get(get_deployment_handler).post(upsert_deployment_handler),
        )
}
