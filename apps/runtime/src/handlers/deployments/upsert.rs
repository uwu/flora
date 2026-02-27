use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    bundler::{DeploymentFile, SourceMapMode, bundle_files_with_sourcemap_mode},
    deployments::Deployment,
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

/// Body for creating or replacing a deployment.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentRequest {
    /// Entry point path for the bundle (e.g. src/main.ts).
    pub entry: String,
    /// Files included in this deployment.
    pub files: Vec<DeploymentFile>,
    /// How to emit source maps for the bundled output.
    #[serde(default)]
    pub source_map_mode: SourceMapMode,
}

/// API representation of a deployment.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentResponse {
    pub guild_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub entry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<String>,
}

impl From<Deployment> for DeploymentResponse {
    fn from(value: Deployment) -> Self {
        Self {
            guild_id: value.guild_id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            entry: value.entry,
            files: None,
            bundle: None,
        }
    }
}

impl DeploymentResponse {
    pub fn with_files(mut self, files: Vec<DeploymentFile>) -> Self {
        self.files = Some(files);
        self
    }

    pub fn with_bundle(mut self, bundle: String) -> Self {
        self.bundle = Some(bundle);
        self
    }
}

/// Create or update a deployment for a guild.
#[utoipa::path(
    post,
    path = "/{guild_id}",
    request_body = DeploymentRequest,
    params(
        ("guild_id" = String, Path, description = "Discord guild id")
    ),
    tag = "deployment",
    responses(
        (status = 200, description = "Deployment stored", body = DeploymentResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn upsert_deployment_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DeploymentRequest>,
) -> Result<ApiJson<DeploymentResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let bundle_name = format!("guild:{guild_id}.bundle.js");
    let bundled = bundle_files_with_sourcemap_mode(
        &bundle_name,
        &request.entry,
        &request.files,
        request.source_map_mode,
    )
    .map_err(|err| ApiError::bad_request(err.to_string()))?;

    let mut files = request.files;
    if let Some(source_map_file) = bundled.source_map_file {
        files.retain(|file| file.path != source_map_file.path);
        files.push(source_map_file);
    }

    let deployment = state
        .deployments
        .upsert_deployment(guild_id.clone(), request.entry, files, bundled.code)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to upsert deployment");
            ApiError::internal(err)
        })?;

    state
        .runtime
        .deploy_guild_script(deployment.clone())
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to deploy guild script");
            ApiError::internal(err)
        })?;

    Ok(ApiJson(Json(deployment.into())))
}
