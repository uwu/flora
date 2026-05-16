use crate::{
    bundler::{BundleLimits, DeploymentFile, bundle_files},
    handlers::{
        auth::{IdentityContext, ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::deployments::{
        CreateDeploymentRevisionInput, Deployment, DeploymentActorType, DeploymentChangeSummary,
        DeploymentRevision, DeploymentRevisionStatus, DeploymentService, DeploymentSource,
        DeploymentSourceMapFile,
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, header::HeaderName},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{error, warn};
use utoipa::ToSchema;

pub const DEPLOY_SOURCE_HEADER: HeaderName = HeaderName::from_static("x-flora-deploy-source");

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentRequest {
    pub entry: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_map: Option<DeploymentSourceMapFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
}

/// Deployment snapshot stored for a guild.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentResponse {
    /// Guild ID for the deployment.
    pub guild_id: String,
    /// Snapshot creation time in RFC3339 (UTC).
    pub created_at: String,
    /// Snapshot update time in RFC3339 (UTC).
    pub updated_at: String,
    /// Entry file path used for the deployment.
    pub entry: String,
    /// Raw files when the deployment was uploaded as sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
    /// Source map included with the bundle, if provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<DeploymentSourceMapFile>,
    /// Bundled output when stored, if included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentActorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub actor_type: DeploymentActorType,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeploymentRevisionResponse {
    pub id: String,
    pub guild_id: String,
    pub entry: String,
    pub status: DeploymentRevisionStatus,
    /// Deployment time in RFC3339 (UTC).
    pub deployed_at: String,
    pub deploy_source: DeploymentSource,
    pub actor: DeploymentActorResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_revision_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_summary: Option<DeploymentChangeSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<DeploymentSourceMapFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<String>,
}

impl DeploymentRevisionResponse {
    pub fn from_revision(revision: DeploymentRevision, include_bundle: bool) -> Self {
        let bundle = if include_bundle {
            Some(revision.bundle)
        } else {
            None
        };

        Self {
            id: revision.id.to_string(),
            guild_id: revision.guild_id,
            entry: revision.entry,
            status: revision.status,
            deployed_at: revision.deployed_at.to_rfc3339(),
            deploy_source: revision.deploy_source,
            actor: DeploymentActorResponse {
                user_id: revision.actor_user_id,
                username: revision.actor_username,
                actor_type: revision.actor_type,
            },
            error_message: revision.error_message,
            build_id: revision.build_id,
            base_revision_id: revision.base_revision_id.map(|value| value.to_string()),
            change_summary: revision.change_summary,
            files: revision.files,
            source_map: revision.source_map,
            bundle,
        }
    }
}

impl From<Deployment> for DeploymentResponse {
    fn from(value: Deployment) -> Self {
        Self {
            guild_id: value.guild_id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            entry: value.entry,
            files: None,
            source_map: None,
            bundle: None,
        }
    }
}

impl DeploymentResponse {
    pub fn with_files(mut self, files: Option<Vec<DeploymentFile>>) -> Self {
        self.files = files;
        self
    }

    pub fn with_source_map(mut self, source_map: Option<DeploymentSourceMapFile>) -> Self {
        self.source_map = source_map;
        self
    }

    pub fn with_bundle(mut self, bundle: String) -> Self {
        self.bundle = Some(bundle);
        self
    }
}

#[derive(Debug, Clone)]
pub struct DeploymentActor {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub actor_type: DeploymentActorType,
}

pub fn list_deploy_source_values() -> &'static str {
    "cli|webui|bootstrap|api|unknown"
}

pub fn parse_deploy_source(headers: &HeaderMap) -> Result<DeploymentSource, ApiError> {
    let source = headers.get(&DEPLOY_SOURCE_HEADER);
    let Some(source) = source else {
        return Ok(DeploymentSource::Unknown);
    };

    let source = source
        .to_str()
        .map_err(|_| ApiError::bad_request("`x-flora-deploy-source` must be ASCII"))?;
    DeploymentSource::from_str(source).map_err(|_| {
        ApiError::bad_request(format!(
            "`x-flora-deploy-source` must be one of {}",
            list_deploy_source_values()
        ))
    })
}

pub async fn actor_from_identity(
    state: &AppState,
    identity: &IdentityContext,
    guild_id: &str,
) -> DeploymentActor {
    let Some(session) = identity.session.as_ref() else {
        let username = resolve_token_username(state, guild_id, &identity.user_id).await;
        return DeploymentActor {
            user_id: Some(identity.user_id.clone()),
            username,
            actor_type: DeploymentActorType::Token,
        };
    };

    DeploymentActor {
        user_id: Some(identity.user_id.clone()),
        username: Some(session.user.username.clone()),
        actor_type: DeploymentActorType::Session,
    }
}

async fn resolve_token_username(state: &AppState, guild_id: &str, user_id: &str) -> Option<String> {
    let Ok(guild_id_num) = guild_id.parse::<u64>() else {
        warn!(target: "flora:api", guild_id, "invalid guild id for deployment actor lookup");
        return None;
    };

    let Ok(user_id_num) = user_id.parse::<u64>() else {
        warn!(target: "flora:api", user_id, "invalid user id for deployment actor lookup");
        return None;
    };

    match state
        .http
        .get_member(guild_id_num.into(), user_id_num.into())
        .await
    {
        Ok(member) => Some(member.user.name.to_string()),
        Err(err) => {
            warn!(
                target: "flora:api",
                guild_id,
                user_id,
                ?err,
                "failed to fetch deployment actor username"
            );
            None
        }
    }
}

fn validate_request(request: &DeploymentRequest) -> Result<(), ApiError> {
    if request.entry.trim().is_empty() {
        return Err(ApiError::bad_request("`entry` must not be empty"));
    }

    match (&request.files, &request.bundle) {
        (Some(files), _) if files.is_empty() => {
            return Err(ApiError::bad_request("`files` must not be empty"));
        }
        (Some(files), _) => {
            for file in files {
                if file.path.trim().is_empty() {
                    return Err(ApiError::bad_request("`files[].path` must not be empty"));
                }
                if file.contents.trim().is_empty() {
                    return Err(ApiError::bad_request(
                        "`files[].contents` must not be empty",
                    ));
                }
            }
        }
        (None, Some(bundle)) if bundle.trim().is_empty() => {
            return Err(ApiError::bad_request("`bundle` must not be empty"));
        }
        (None, None) => {
            return Err(ApiError::bad_request(
                "Either `files` or `bundle` is required",
            ));
        }
        _ => {}
    }

    if let Some(source_map) = &request.source_map {
        if source_map.path.trim().is_empty() {
            return Err(ApiError::bad_request("`source_map.path` must not be empty"));
        }

        if source_map.contents.trim().is_empty() {
            return Err(ApiError::bad_request(
                "`source_map.contents` must not be empty",
            ));
        }

        if !source_map.path.ends_with(".map") {
            return Err(ApiError::bad_request(
                "`source_map.path` must end with `.map`",
            ));
        }
    }

    Ok(())
}

#[utoipa::path(
    post,
    path = "/{guild_id}",
    request_body = DeploymentRequest,
    params(
        ("guild_id" = String, Path, description = "Guild ID")
    ),
    tag = "Deployments",
    summary = "Deploy a guild",
    description = "Creates or updates the active deployment for a guild, then records a revision.",
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

    let deploy_source = parse_deploy_source(&headers)?;
    let actor = actor_from_identity(&state, &identity, &guild_id).await;

    let files = request.files;
    let (bundle, source_map) = if let Some(bundle) = request.bundle {
        (bundle, request.source_map)
    } else if let Some(files_ref) = files.as_ref() {
        let bundle_name = format!("guild:{guild_id}.bundle.js");
        let bundled = bundle_files(
            &bundle_name,
            &request.entry,
            files_ref,
            BundleLimits::default(),
        )
        .map_err(|err| ApiError::bad_request(format!("invalid deployment files: {err}")))?;
        (bundled.code, None)
    } else {
        let Some(bundle) = request.bundle else {
            return Err(ApiError::bad_request(
                "bundle is required when files are missing",
            ));
        };
        (bundle, request.source_map)
    };

    let previous_success = state
        .deployments
        .get_previous_successful_revision(&guild_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to fetch previous successful revision");
            ApiError::internal(err)
        })?;

    let base_revision_id = previous_success.as_ref().map(|row| row.revision_id);
    let base_files = previous_success.as_ref().and_then(|row| row.files.as_ref());
    let change_summary = DeploymentService::summarize_changes(files.as_ref(), base_files);

    let now = Utc::now();
    let deployment = Deployment {
        guild_id: guild_id.clone(),
        entry: request.entry,
        files: files.clone(),
        source_map: source_map.clone(),
        bundle: bundle.clone(),
        created_at: now,
        updated_at: now,
    };

    let deploy_result = state.runtime.deploy_guild_script(deployment.clone()).await;
    let status = match &deploy_result {
        Ok(_) => DeploymentRevisionStatus::Success,
        Err(_) => DeploymentRevisionStatus::Failed,
    };
    let error_message = deploy_result.as_ref().err().map(|err| err.to_string());

    state
        .deployments
        .create_revision(CreateDeploymentRevisionInput {
            guild_id: guild_id.clone(),
            entry: deployment.entry.clone(),
            files: deployment.files.clone(),
            bundle: deployment.bundle.clone(),
            source_map: deployment.source_map.clone(),
            status,
            deploy_source,
            actor_user_id: actor.user_id,
            actor_username: actor.username,
            actor_type: actor.actor_type,
            error_message,
            build_id: request.build_id,
            base_revision_id,
            change_summary,
        })
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to create deployment revision");
            ApiError::internal(err)
        })?;

    deploy_result.map_err(|err| {
        error!(target: "flora:api", guild_id, ?err, "failed to deploy guild script");
        ApiError::bad_request(format!("deployment failed: {err}"))
    })?;

    let deployment = state
        .deployments
        .upsert_deployment(
            deployment.guild_id,
            deployment.entry,
            deployment.files,
            deployment.bundle,
            deployment.source_map,
        )
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to update deployment snapshot");
            ApiError::internal(err)
        })?;

    Ok(ApiJson(Json(deployment.into())))
}

#[cfg(test)]
mod tests {
    use super::{DeploymentRequest, parse_deploy_source, validate_request};
    use crate::{
        bundler::DeploymentFile,
        handlers::error::ApiError,
        services::deployments::{DeploymentSource, DeploymentSourceMapFile},
    };
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn validate_request_rejects_empty_bundle() {
        let request = DeploymentRequest {
            entry: "src/main.ts".to_string(),
            files: None,
            bundle: Some("   ".to_string()),
            source_map: None,
            build_id: None,
        };

        let result = validate_request(&request);
        assert!(matches!(result, Err(ApiError::BadRequest { .. })));
    }

    #[test]
    fn validate_request_rejects_invalid_source_map() {
        let request = DeploymentRequest {
            entry: "src/main.ts".to_string(),
            files: None,
            bundle: Some("console.log('ok')".to_string()),
            source_map: Some(DeploymentSourceMapFile {
                path: "source-map.txt".to_string(),
                contents: "{}".to_string(),
            }),
            build_id: None,
        };

        let result = validate_request(&request);
        assert!(matches!(result, Err(ApiError::BadRequest { .. })));
    }

    #[test]
    fn validate_request_accepts_bundle_with_source_map() {
        let request = DeploymentRequest {
            entry: "src/main.ts".to_string(),
            files: None,
            bundle: Some("console.log('ok')".to_string()),
            source_map: Some(DeploymentSourceMapFile {
                path: "bundle.js.map".to_string(),
                contents: "{}".to_string(),
            }),
            build_id: None,
        };

        validate_request(&request).expect("request should be valid");
    }

    #[test]
    fn validate_request_accepts_files_without_bundle() {
        let request = DeploymentRequest {
            entry: "src/main.ts".to_string(),
            files: Some(vec![DeploymentFile {
                path: "src/main.ts".to_string(),
                contents: "export default 1".to_string(),
            }]),
            bundle: None,
            source_map: None,
            build_id: None,
        };

        validate_request(&request).expect("request should be valid");
    }

    #[test]
    fn parse_deploy_source_defaults_to_unknown() {
        let headers = HeaderMap::new();
        let source = parse_deploy_source(&headers).expect("source");
        assert!(matches!(source, DeploymentSource::Unknown));
    }

    #[test]
    fn parse_deploy_source_reads_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-flora-deploy-source", HeaderValue::from_static("cli"));
        let source = parse_deploy_source(&headers).expect("source");
        assert!(matches!(source, DeploymentSource::Cli));
    }
}
