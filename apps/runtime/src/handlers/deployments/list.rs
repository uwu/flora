use axum::{Json, extract::State, http::HeaderMap};
use tracing::error;

use super::DeploymentResponse;
use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

/// List every stored deployment.
#[utoipa::path(
    get,
    path = "/",
    tag = "deployment",
    responses(
        (status = 200, description = "Deployments retrieved", body = [DeploymentResponse]),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn list_deployments_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<Vec<DeploymentResponse>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;

    let deployments = state.deployments.list_deployments().await.map_err(|err| {
        error!(target: "flora:api", ?err, "failed to list deployments");
        ApiError::internal(err)
    })?;

    let mut response = Vec::new();
    for deployment in deployments {
        if ensure_guild_admin(&state, &identity, &deployment.guild_id)
            .await
            .is_ok()
        {
            response.push(DeploymentResponse::from(deployment));
        }
    }
    Ok(ApiJson(Json(response)))
}
