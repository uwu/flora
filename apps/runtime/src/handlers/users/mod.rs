use axum::{Router, routing::get};
use utoipa::OpenApi;

use crate::state::AppState;

pub mod deployment;

#[derive(OpenApi)]
#[openapi(
    paths(
        deployment::upsert_user_deployment_handler,
        deployment::get_user_deployment_handler,
        deployment::delete_user_deployment_handler
    ),
    components(schemas(
        crate::handlers::deployments::DeploymentRequest,
        crate::handlers::deployments::DeploymentResponse,
        crate::handlers::error::ErrorResponse
    )),
    tags((name = "user", description = "User deployment management"))
)]
pub struct UserApi;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/@me/deployment",
        get(deployment::get_user_deployment_handler)
            .post(deployment::upsert_user_deployment_handler)
            .delete(deployment::delete_user_deployment_handler),
    )
}
