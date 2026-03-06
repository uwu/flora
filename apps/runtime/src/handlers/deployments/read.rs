use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
pub struct DeploymentQuery {
    #[serde(default)]
    include_bundle: bool,
}

/// Fetch a single deployment by guild id.
#[utoipa::path(
    get,
    path = "/{guild_id}",
    params(
        ("guild_id" = String, Path, description = "Discord guild id"),
        ("include_bundle" = Option<bool>, Query, description = "Include bundled output in response")
    ),
    tag = "deployment",
    responses(
        (status = 200, description = "Deployment found", body = DeploymentResponse),
        (status = 404, description = "Deployment not found", body = crate::handlers::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn get_deployment_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DeploymentQuery>,
) -> Result<ApiJson<DeploymentResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;

    let deployment = state
        .deployments
        .get_deployment(&guild_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to fetch deployment");
            ApiError::internal(err)
        })?;

    let Some(deployment) = deployment else {
        return Err(ApiError::not_found("deployment not found"));
    };

    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let files = deployment.files.clone();
    let source_map = deployment.source_map.clone();
    let bundle = deployment.bundle.clone();
    let mut response = DeploymentResponse::from(deployment)
        .with_files(files)
        .with_source_map(source_map);
    if query.include_bundle {
        response = response.with_bundle(bundle);
    }
    Ok(ApiJson(Json(response)))
}
