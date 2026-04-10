use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;

use crate::state::AppState;

pub mod history;
pub mod list;
pub mod read;
pub mod revision;
pub mod rollback;
pub mod upsert;

pub use history::list_deployment_history_handler;
pub use list::{DeploymentListItem, list_deployments_handler};
pub use read::get_deployment_handler;
pub use revision::get_deployment_revision_handler;
pub use rollback::rollback_deployment_handler;
pub use upsert::{
    DeploymentActorResponse, DeploymentRequest, DeploymentResponse, DeploymentRevisionResponse,
    list_deploy_source_values, parse_deploy_source, upsert_deployment_handler,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        list::list_deployments_handler,
        read::get_deployment_handler,
        upsert::upsert_deployment_handler,
        history::list_deployment_history_handler,
        revision::get_deployment_revision_handler,
        rollback::rollback_deployment_handler
    ),
    components(
        schemas(
            DeploymentListItem,
            DeploymentRequest,
            DeploymentResponse,
            DeploymentRevisionResponse,
            DeploymentActorResponse,
            super::error::ErrorResponse
        )
    ),
    tags((name = "Deployments", description = "Manage per-guild bot deployments"))
)]
pub struct DeploymentApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_deployments_handler))
        .route(
            "/{guild_id}",
            get(get_deployment_handler).post(upsert_deployment_handler),
        )
        .route("/{guild_id}/history", get(list_deployment_history_handler))
        .route(
            "/{guild_id}/revisions/{revision_id}",
            get(get_deployment_revision_handler),
        )
        .route(
            "/{guild_id}/rollback/{revision_id}",
            post(rollback_deployment_handler),
        )
}
