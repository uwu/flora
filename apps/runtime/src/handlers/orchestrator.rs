use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    routing::{delete, get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::{OpenApi, ToSchema};

use crate::{
    bundler::{BundleLimits, DeploymentFile, bundle_files},
    handlers::{
        auth::{IdentityContext, require_identity},
        deployments::upsert::{DeploymentRequest, parse_deploy_source, validate_request},
        error::ApiError,
        response::ApiJson,
    },
    services::{
        deployments::{
            CreateDeploymentRevisionInput, Deployment, DeploymentActorType,
            DeploymentRevisionStatus, DeploymentService, DeploymentSourceMapFile,
        },
        orchestrator::ORCHESTRATOR_DEPLOYMENT_ID,
    },
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(get_orchestrator_deployment, deploy_orchestrator, delete_orchestrator),
    components(schemas(OrchestratorDeploymentResponse, DeploymentRequest)),
    tags((name = "Orchestrator", description = "Trusted operator-owned feature authorization deployment"))
)]
pub struct OrchestratorApi;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/deployment", get(get_orchestrator_deployment))
        .route("/deployment", post(deploy_orchestrator))
        .route("/deployment", delete(delete_orchestrator))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct OrchestratorDeploymentResponse {
    pub entry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<DeploymentFile>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map: Option<DeploymentSourceMapFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Deployment> for OrchestratorDeploymentResponse {
    fn from(deployment: Deployment) -> Self {
        Self {
            entry: deployment.entry,
            files: deployment.files,
            source_map: deployment.source_map,
            bundle: Some(deployment.bundle),
            created_at: deployment.created_at.to_rfc3339(),
            updated_at: deployment.updated_at.to_rfc3339(),
        }
    }
}

async fn require_operator(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<IdentityContext, ApiError> {
    let identity = require_identity(state, headers).await?;
    let Some(operator_user_id) = state.orchestrator_operator_user_id.as_deref() else {
        return Err(ApiError::forbidden("orchestrator management is disabled"));
    };
    if identity.user_id != operator_user_id {
        return Err(ApiError::forbidden(
            "only the configured orchestrator operator may manage this deployment",
        ));
    }
    Ok(identity)
}

fn actor(identity: &IdentityContext) -> (Option<String>, Option<String>, DeploymentActorType) {
    let username = identity
        .session
        .as_ref()
        .map(|session| session.user.username.clone());
    let actor_type = if identity.session.is_some() {
        DeploymentActorType::Session
    } else {
        DeploymentActorType::Token
    };
    (Some(identity.user_id.clone()), username, actor_type)
}

#[utoipa::path(
    get,
    path = "/deployment",
    tag = "Orchestrator",
    summary = "Get the orchestrator deployment",
    description = "Returns the active trusted orchestrator deployment. Only the configured Discord operator may access this endpoint; the deployment may contain sensitive feature-policy source.",
    responses(
        (status = 200, description = "Active orchestrator deployment", body = OrchestratorDeploymentResponse),
        (status = 404, description = "No orchestrator deployment", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn get_orchestrator_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<OrchestratorDeploymentResponse>, ApiError> {
    require_operator(&state, &headers).await?;
    let deployment = state
        .deployments
        .get_deployment(ORCHESTRATOR_DEPLOYMENT_ID)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("orchestrator deployment not found"))?;
    Ok(ApiJson(Json(deployment.into())))
}

#[utoipa::path(
    post,
    path = "/deployment",
    tag = "Orchestrator",
    summary = "Deploy the orchestrator",
    description = "Validates and atomically replaces the trusted orchestrator isolate, persists a deployment revision, and immediately reconciles User Bot and Server Custom Bot gateways against the new feature policy. A failed deployment leaves the previous orchestrator active.",
    request_body = DeploymentRequest,
    responses(
        (status = 200, description = "Orchestrator deployed", body = OrchestratorDeploymentResponse),
        (status = 400, description = "Invalid deployment", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn deploy_orchestrator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DeploymentRequest>,
) -> Result<ApiJson<OrchestratorDeploymentResponse>, ApiError> {
    let identity = require_operator(&state, &headers).await?;
    validate_request(&request)?;

    let deploy_source = parse_deploy_source(&headers)?;
    let (actor_user_id, actor_username, actor_type) = actor(&identity);
    let files = request.files;
    let build_id = request.build_id;
    let (bundle, source_map) = if let Some(bundle) = request.bundle {
        (bundle, request.source_map)
    } else {
        let files_ref = files
            .as_ref()
            .ok_or_else(|| ApiError::bad_request("files or bundle are required"))?;
        let bundled = bundle_files(
            "orchestrator.bundle.js",
            &request.entry,
            files_ref,
            BundleLimits::default(),
        )
        .map_err(|err| ApiError::bad_request(format!("invalid deployment files: {err}")))?;
        (bundled.code, None)
    };

    let previous = state
        .deployments
        .get_persisted_deployment(ORCHESTRATOR_DEPLOYMENT_ID)
        .await
        .map_err(ApiError::internal)?;
    let previous_revision = state
        .deployments
        .get_previous_successful_revision(ORCHESTRATOR_DEPLOYMENT_ID)
        .await
        .map_err(ApiError::internal)?;
    let now = Utc::now();
    let candidate = Deployment {
        guild_id: ORCHESTRATOR_DEPLOYMENT_ID.to_string(),
        entry: request.entry,
        files: files.clone(),
        source_map: source_map.clone(),
        bundle: bundle.clone(),
        created_at: previous.as_ref().map_or(now, |item| item.created_at),
        updated_at: now,
    };

    if let Err(err) = state
        .runtime
        .deploy_orchestrator_script(candidate.clone())
        .await
    {
        let _ = state
            .deployments
            .create_revision(CreateDeploymentRevisionInput {
                guild_id: ORCHESTRATOR_DEPLOYMENT_ID.to_string(),
                entry: candidate.entry.clone(),
                files,
                bundle,
                source_map,
                status: DeploymentRevisionStatus::Failed,
                deploy_source,
                actor_user_id,
                actor_username,
                actor_type,
                error_message: Some(err.to_string()),
                build_id,
                base_revision_id: previous_revision
                    .as_ref()
                    .map(|revision| revision.revision_id),
                change_summary: DeploymentService::summarize_changes(
                    candidate.files.as_ref(),
                    previous_revision
                        .as_ref()
                        .and_then(|revision| revision.files.as_ref()),
                ),
            })
            .await;
        return Err(ApiError::bad_request(format!(
            "orchestrator deployment failed: {err}"
        )));
    }

    let persisted = state
        .deployments
        .create_successful_revision_and_upsert_deployment(CreateDeploymentRevisionInput {
            guild_id: ORCHESTRATOR_DEPLOYMENT_ID.to_string(),
            entry: candidate.entry.clone(),
            files: candidate.files.clone(),
            bundle: candidate.bundle.clone(),
            source_map: candidate.source_map.clone(),
            status: DeploymentRevisionStatus::Success,
            deploy_source,
            actor_user_id,
            actor_username,
            actor_type,
            error_message: None,
            build_id,
            base_revision_id: previous_revision
                .as_ref()
                .map(|revision| revision.revision_id),
            change_summary: DeploymentService::summarize_changes(
                candidate.files.as_ref(),
                previous_revision
                    .as_ref()
                    .and_then(|revision| revision.files.as_ref()),
            ),
        })
        .await;

    let (_, deployment) = match persisted {
        Ok(value) => value,
        Err(err) => {
            error!(target: "flora:api", ?err, "failed to persist orchestrator deployment");
            match previous {
                Some(previous) => state
                    .runtime
                    .deploy_orchestrator_script(previous)
                    .await
                    .map_err(ApiError::internal)?,
                None => state
                    .runtime
                    .undeploy_orchestrator_script()
                    .await
                    .map_err(ApiError::internal)?,
            }
            return Err(ApiError::internal(err));
        }
    };

    if let Err(err) = state.custom_bot_gateway.reconcile_all().await {
        error!(target: "flora:api", ?err, "failed to reconcile custom bots after orchestrator deploy");
    }
    if let Err(err) = state.server_custom_bot_gateway.reconcile_all().await {
        error!(target: "flora:api", ?err, "failed to reconcile server custom bots after orchestrator deploy");
    }

    Ok(ApiJson(Json(deployment.into())))
}

#[utoipa::path(
    delete,
    path = "/deployment",
    tag = "Orchestrator",
    summary = "Delete the orchestrator deployment",
    description = "Removes the trusted orchestrator deployment and restores deny-by-default feature authorization. Existing User Bot and Server Custom Bot gateways are reconciled immediately without deleting their encrypted configuration.",
    responses((status = 200, description = "Orchestrator deleted"))
)]
pub async fn delete_orchestrator(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<()>, ApiError> {
    require_operator(&state, &headers).await?;
    let previous = state
        .deployments
        .get_persisted_deployment(ORCHESTRATOR_DEPLOYMENT_ID)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("orchestrator deployment not found"))?;

    state
        .runtime
        .undeploy_orchestrator_script()
        .await
        .map_err(ApiError::internal)?;
    if let Err(err) = state
        .deployments
        .delete_deployment(ORCHESTRATOR_DEPLOYMENT_ID)
        .await
    {
        state
            .runtime
            .deploy_orchestrator_script(previous)
            .await
            .map_err(ApiError::internal)?;
        return Err(ApiError::internal(err));
    }
    if let Err(err) = state.custom_bot_gateway.reconcile_all().await {
        error!(target: "flora:api", ?err, "failed to reconcile custom bots after orchestrator delete");
    }
    if let Err(err) = state.server_custom_bot_gateway.reconcile_all().await {
        error!(target: "flora:api", ?err, "failed to reconcile server custom bots after orchestrator delete");
    }
    Ok(ApiJson(Json(())))
}
