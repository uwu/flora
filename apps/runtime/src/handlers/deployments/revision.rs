use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use super::DeploymentRevisionResponse;
use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct DeploymentRevisionQuery {
    #[serde(default)]
    include_bundle: bool,
}

#[utoipa::path(
    get,
    path = "/{guild_id}/revisions/{revision_id}",
    params(
        ("guild_id" = String, Path, description = "Discord guild id"),
        ("revision_id" = String, Path, description = "Revision id"),
        ("include_bundle" = Option<bool>, Query, description = "Include bundled output in response")
    ),
    tag = "deployment",
    responses(
        (status = 200, description = "Revision found", body = DeploymentRevisionResponse),
        (status = 404, description = "Revision not found", body = crate::handlers::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn get_deployment_revision_handler(
    Path((guild_id, revision_id)): Path<(String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DeploymentRevisionQuery>,
) -> Result<ApiJson<DeploymentRevisionResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let revision_id =
        Uuid::parse_str(&revision_id).map_err(|_| ApiError::bad_request("invalid revision id"))?;

    let revision = state
        .deployments
        .get_guild_revision(&guild_id, revision_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to fetch deployment revision");
            ApiError::internal(err)
        })?;

    let Some(revision) = revision else {
        return Err(ApiError::not_found("deployment revision not found"));
    };

    Ok(ApiJson(Json(DeploymentRevisionResponse::from_revision(
        revision,
        query.include_bundle,
    ))))
}
