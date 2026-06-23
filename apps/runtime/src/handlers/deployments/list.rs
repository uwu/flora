use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::deployments::Deployment,
    state::AppState,
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentListItem {
    pub guild_id: String,
    pub entry: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Deployment> for DeploymentListItem {
    fn from(d: Deployment) -> Self {
        Self {
            guild_id: d.guild_id,
            entry: d.entry,
            created_at: d.created_at.to_rfc3339(),
            updated_at: d.updated_at.to_rfc3339(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/",
    tag = "Deployments",
    summary = "List deployments",
    description = "Returns all deployment snapshots the authenticated user has access to.",
    responses(
        (status = 200, description = "Deployments list", body = [DeploymentListItem]),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn list_deployments_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<Vec<DeploymentListItem>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;

    let deployments = state.deployments.list_deployments().await.map_err(|err| {
        error!(target: "flora:api", ?err, "failed to list deployments");
        ApiError::internal(err)
    })?;

    let mut items = Vec::with_capacity(deployments.len());
    for deployment in deployments {
        match ensure_guild_admin(&state, &identity, &deployment.guild_id).await {
            Ok(()) => {}
            Err(ApiError::Forbidden { .. }) => continue,
            Err(err) => return Err(err),
        }

        items.push(DeploymentListItem::from(deployment));
    }

    Ok(ApiJson(Json(items)))
}
