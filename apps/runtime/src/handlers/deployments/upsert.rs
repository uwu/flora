use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    deployments::{Deployment, DeploymentSourceMapFile},
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
    /// Prebuilt JavaScript bundle source.
    pub bundle: String,
    /// Optional source map file for the prebuilt bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_map: Option<DeploymentSourceMapFile>,
}

/// API representation of a deployment.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentResponse {
    pub guild_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub entry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<DeploymentSourceMapFile>,
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
            source_map: None,
            bundle: None,
        }
    }
}

impl DeploymentResponse {
    pub fn with_source_map(mut self, source_map: Option<DeploymentSourceMapFile>) -> Self {
        self.source_map = source_map;
        self
    }

    pub fn with_bundle(mut self, bundle: String) -> Self {
        self.bundle = Some(bundle);
        self
    }
}

fn validate_request(request: &DeploymentRequest) -> Result<(), ApiError> {
    if request.entry.trim().is_empty() {
        return Err(ApiError::bad_request("entry must not be empty"));
    }

    if request.bundle.trim().is_empty() {
        return Err(ApiError::bad_request("bundle must not be empty"));
    }

    if let Some(source_map) = &request.source_map {
        if source_map.path.trim().is_empty() {
            return Err(ApiError::bad_request("source_map.path must not be empty"));
        }

        if source_map.contents.trim().is_empty() {
            return Err(ApiError::bad_request(
                "source_map.contents must not be empty",
            ));
        }

        if !source_map.path.ends_with(".map") {
            return Err(ApiError::bad_request("source_map.path must end with .map"));
        }
    }

    Ok(())
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

    validate_request(&request)?;

    let deployment = state
        .deployments
        .upsert_deployment(
            guild_id.clone(),
            request.entry,
            request.bundle,
            request.source_map,
        )
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

#[cfg(test)]
mod tests {
    use super::{DeploymentRequest, validate_request};
    use crate::{deployments::DeploymentSourceMapFile, handlers::error::ApiError};

    #[test]
    fn validate_request_rejects_empty_bundle() {
        let request = DeploymentRequest {
            entry: "src/main.ts".to_string(),
            bundle: "   ".to_string(),
            source_map: None,
        };

        let result = validate_request(&request);
        assert!(matches!(result, Err(ApiError::BadRequest { .. })));
    }

    #[test]
    fn validate_request_rejects_invalid_source_map() {
        let request = DeploymentRequest {
            entry: "src/main.ts".to_string(),
            bundle: "console.log('ok')".to_string(),
            source_map: Some(DeploymentSourceMapFile {
                path: "source-map.txt".to_string(),
                contents: "{}".to_string(),
            }),
        };

        let result = validate_request(&request);
        assert!(matches!(result, Err(ApiError::BadRequest { .. })));
    }

    #[test]
    fn validate_request_accepts_bundle_with_source_map() {
        let request = DeploymentRequest {
            entry: "src/main.ts".to_string(),
            bundle: "console.log('ok')".to_string(),
            source_map: Some(DeploymentSourceMapFile {
                path: "bundle.js.map".to_string(),
                contents: "{}".to_string(),
            }),
        };

        validate_request(&request).expect("request should be valid");
    }
}
