use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    bundler::{BundleLimits, DeploymentFile, bundle_files},
    handlers::{
        auth::{IdentityContext, require_identity},
        deployments::upsert::{DeploymentRequest, parse_deploy_source, validate_request},
        error::ApiError,
        response::ApiJson,
    },
    services::{
        custom_bots::{
            CustomBot, UserBotLimitReached, custom_bot_deployment_id,
            custom_bot_id_from_deployment_id,
        },
        deployments::{
            CreateDeploymentRevisionInput, Deployment, DeploymentActorType,
            DeploymentRevisionStatus, DeploymentService, DeploymentSourceMapFile,
        },
        orchestrator::FeatureAuthorizationRequest,
    },
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        list_custom_bots,
        create_custom_bot,
        get_custom_bot,
        delete_custom_bot,
        get_custom_bot_deployment,
        deploy_custom_bot
    ),
    components(schemas(
        CustomBotResponse,
        CreateCustomBotRequest,
        CustomBotDeploymentResponse,
        DeploymentRequest
    )),
    tags((name = "User Bots", description = "User-owned Discord applications with independent flora deployments"))
)]
pub struct CustomBotsApi;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(list_custom_bots))
        .route("/", post(create_custom_bot))
        .route("/{bot_id}", get(get_custom_bot))
        .route("/{bot_id}", delete(delete_custom_bot))
        .route("/{bot_id}/deployment", get(get_custom_bot_deployment))
        .route("/{bot_id}/deployment", post(deploy_custom_bot))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomBotRequest {
    #[serde(default)]
    pub label: Option<String>,
    /// Discord bot token. The token is validated, encrypted, and never returned.
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomBotResponse {
    pub id: String,
    pub label: Option<String>,
    pub bot_user_id: String,
    pub bot_username: String,
    pub application_id: String,
    pub has_deployment: bool,
    pub running: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomBotDeploymentResponse {
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

impl From<Deployment> for CustomBotDeploymentResponse {
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

pub async fn require_feature_identity(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<IdentityContext, ApiError> {
    let identity = require_identity(state, headers).await?;
    ensure_feature_enabled(state, &identity).await?;
    Ok(identity)
}

pub async fn ensure_feature_enabled(
    state: &AppState,
    identity: &IdentityContext,
) -> Result<(), ApiError> {
    let allowed = state
        .runtime
        .authorize_feature(FeatureAuthorizationRequest {
            feature: "custom_bots".to_string(),
            user_id: identity.user_id.clone(),
            metadata: None,
        })
        .await
        .unwrap_or(false);
    if !allowed {
        return Err(ApiError::feature_disabled(
            "User Bots are not enabled for your account. Ask a flora operator to enable the feature before trying again",
        ));
    }
    Ok(())
}

pub async fn ensure_scope_owner(
    state: &AppState,
    identity: &IdentityContext,
    scope_id: &str,
) -> Result<CustomBot, ApiError> {
    let bot_id = custom_bot_id_from_deployment_id(scope_id)
        .ok_or_else(|| ApiError::bad_request("invalid custom bot deployment scope"))?;
    ensure_bot_owner(state, identity, bot_id).await
}

async fn ensure_bot_owner(
    state: &AppState,
    identity: &IdentityContext,
    bot_id: Uuid,
) -> Result<CustomBot, ApiError> {
    let bot = state
        .custom_bots
        .get(bot_id)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("custom bot not found"))?;
    if bot.owner_user_id != identity.user_id {
        return Err(ApiError::forbidden(
            "This User Bot belongs to another account and cannot be managed by the current user",
        ));
    }
    Ok(bot)
}

async fn response(state: &AppState, bot: CustomBot) -> Result<CustomBotResponse, ApiError> {
    let deployment_id = custom_bot_deployment_id(bot.id);
    let has_deployment = state
        .deployments
        .get_persisted_deployment(&deployment_id)
        .await
        .map_err(ApiError::internal)?
        .is_some();
    Ok(CustomBotResponse {
        id: bot.id.to_string(),
        label: bot.label,
        bot_user_id: bot.bot_user_id,
        bot_username: bot.bot_username,
        application_id: bot.application_id,
        has_deployment,
        running: state.custom_bot_gateway.is_running(bot.id),
        created_at: bot.created_at.to_rfc3339(),
        updated_at: bot.updated_at.to_rfc3339(),
    })
}

#[utoipa::path(
    get,
    path = "/",
    tag = "User Bots",
    summary = "List User Bots",
    description = "Returns every User Bot owned by the authenticated user, including deployment and gateway status. The User Bots feature must be enabled for the account.",
    responses((status = 200, description = "User Bots owned by the authenticated user", body = Vec<CustomBotResponse>))
)]
pub async fn list_custom_bots(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<Vec<CustomBotResponse>>, ApiError> {
    let identity = require_feature_identity(&state, &headers).await?;
    let bots = state
        .custom_bots
        .list_for_owner(&identity.user_id)
        .await
        .map_err(ApiError::internal)?;
    let mut responses = Vec::with_capacity(bots.len());
    for bot in bots {
        responses.push(response(&state, bot).await?);
    }
    Ok(ApiJson(Json(responses)))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "User Bots",
    summary = "Create a User Bot",
    description = "Validates a Discord bot token, encrypts it at rest, and creates a User Bot owned by the authenticated user. Each account may own one User Bot; creating another returns a conflict error until the existing User Bot is deleted. The plaintext token is never returned. A deployment must be uploaded separately before the gateway starts.",
    request_body = CreateCustomBotRequest,
    responses(
        (status = 200, description = "User Bot created", body = CustomBotResponse),
        (status = 409, description = "Account already owns a User Bot", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn create_custom_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateCustomBotRequest>,
) -> Result<ApiJson<CustomBotResponse>, ApiError> {
    let identity = require_feature_identity(&state, &headers).await?;
    let bot = state
        .custom_bots
        .create(&identity.user_id, request.label, &request.token)
        .await
        .map_err(|err| match err.downcast_ref::<UserBotLimitReached>() {
            Some(_) => ApiError::conflict(
                "Your account already has a User Bot. Delete the existing User Bot before creating a new one",
            ),
            None => ApiError::bad_request(err.to_string()),
        })?;
    Ok(ApiJson(Json(response(&state, bot).await?)))
}

#[utoipa::path(
    get,
    path = "/{bot_id}",
    tag = "User Bots",
    summary = "Get a User Bot",
    description = "Returns metadata and runtime status for a User Bot owned by the authenticated user. Tokens and other encrypted credentials are never included.",
    params(("bot_id" = String, Path, description = "User Bot ID")),
    responses((status = 200, description = "User Bot metadata and status", body = CustomBotResponse))
)]
pub async fn get_custom_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<Uuid>,
) -> Result<ApiJson<CustomBotResponse>, ApiError> {
    let identity = require_feature_identity(&state, &headers).await?;
    let bot = ensure_bot_owner(&state, &identity, bot_id).await?;
    Ok(ApiJson(Json(response(&state, bot).await?)))
}

#[utoipa::path(
    delete,
    path = "/{bot_id}",
    tag = "User Bots",
    summary = "Delete a User Bot",
    description = "Stops the User Bot gateway, removes its active deployment, and deletes its encrypted token metadata. The operation is restricted to the owning user.",
    params(("bot_id" = String, Path, description = "User Bot ID")),
    responses((status = 200, description = "User Bot deleted"))
)]
pub async fn delete_custom_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<Uuid>,
) -> Result<ApiJson<()>, ApiError> {
    let identity = require_feature_identity(&state, &headers).await?;
    ensure_bot_owner(&state, &identity, bot_id).await?;
    state
        .custom_bot_gateway
        .deactivate(bot_id)
        .await
        .map_err(ApiError::internal)?;
    state
        .deployments
        .delete_deployment(&custom_bot_deployment_id(bot_id))
        .await
        .map_err(ApiError::internal)?;
    state
        .custom_bots
        .delete(bot_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(ApiJson(Json(())))
}

#[utoipa::path(
    get,
    path = "/{bot_id}/deployment",
    tag = "User Bots",
    summary = "Get a User Bot deployment",
    description = "Returns the active deployment source and bundle for a User Bot owned by the authenticated user. Returns a not-found problem when the bot has not been deployed.",
    params(("bot_id" = String, Path, description = "User Bot ID")),
    responses((status = 200, description = "Active User Bot deployment", body = CustomBotDeploymentResponse))
)]
pub async fn get_custom_bot_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<Uuid>,
) -> Result<ApiJson<CustomBotDeploymentResponse>, ApiError> {
    let identity = require_feature_identity(&state, &headers).await?;
    ensure_bot_owner(&state, &identity, bot_id).await?;
    let deployment = state
        .deployments
        .get_deployment(&custom_bot_deployment_id(bot_id))
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("custom bot deployment not found"))?;
    Ok(ApiJson(Json(deployment.into())))
}

#[utoipa::path(
    post,
    path = "/{bot_id}/deployment",
    tag = "User Bots",
    summary = "Deploy a User Bot",
    description = "Builds or accepts the supplied bundle, atomically replaces the User Bot isolate, records a deployment revision, and starts its Discord gateway. If deployment fails, the previous runtime and persisted deployment are restored.",
    params(("bot_id" = String, Path, description = "User Bot ID")),
    request_body = DeploymentRequest,
    responses((status = 200, description = "Active User Bot deployment", body = CustomBotDeploymentResponse))
)]
pub async fn deploy_custom_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<Uuid>,
    Json(request): Json<DeploymentRequest>,
) -> Result<ApiJson<CustomBotDeploymentResponse>, ApiError> {
    let identity = require_feature_identity(&state, &headers).await?;
    ensure_bot_owner(&state, &identity, bot_id).await?;
    validate_request(&request)?;
    let deploy_source = parse_deploy_source(&headers)?;
    let actor_username = identity
        .session
        .as_ref()
        .map(|session| session.user.username.clone());
    let actor_type = if identity.session.is_some() {
        DeploymentActorType::Session
    } else {
        DeploymentActorType::Token
    };
    let files = request.files;
    let build_id = request.build_id;
    let (bundle, source_map) = if let Some(bundle) = request.bundle {
        (bundle, request.source_map)
    } else {
        let files_ref = files
            .as_ref()
            .ok_or_else(|| ApiError::bad_request("files or bundle are required"))?;
        let bundled = bundle_files(
            &format!("custom-bot:{bot_id}.bundle.js"),
            &request.entry,
            files_ref,
            BundleLimits::default(),
        )
        .map_err(|err| ApiError::bad_request(format!("invalid deployment files: {err}")))?;
        (bundled.code, None)
    };
    let scope_id = custom_bot_deployment_id(bot_id);
    let previous = state
        .deployments
        .get_persisted_deployment(&scope_id)
        .await
        .map_err(ApiError::internal)?;
    let previous_revision = state
        .deployments
        .get_previous_successful_revision(&scope_id)
        .await
        .map_err(ApiError::internal)?;
    let now = Utc::now();
    let candidate = Deployment {
        guild_id: scope_id.clone(),
        entry: request.entry,
        files: files.clone(),
        source_map: source_map.clone(),
        bundle: bundle.clone(),
        created_at: previous
            .as_ref()
            .map_or(now, |deployment| deployment.created_at),
        updated_at: now,
    };

    if let Err(err) = state
        .custom_bot_gateway
        .activate(bot_id, candidate.clone())
        .await
    {
        let _ = state
            .deployments
            .create_revision(CreateDeploymentRevisionInput {
                guild_id: scope_id.clone(),
                entry: candidate.entry.clone(),
                files,
                bundle,
                source_map,
                status: DeploymentRevisionStatus::Failed,
                deploy_source,
                actor_user_id: Some(identity.user_id.clone()),
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
        if let Some(previous) = previous {
            if let Err(restore_err) = state.custom_bot_gateway.activate(bot_id, previous).await {
                warn!(target: "flora:custom-bots", bot_id = %bot_id, ?restore_err, "failed to restore previous custom bot runtime");
            }
        } else {
            let _ = state.custom_bot_gateway.deactivate(bot_id).await;
        }
        return Err(ApiError::bad_request(format!(
            "custom bot deployment failed: {err}"
        )));
    }

    let persisted = state
        .deployments
        .create_successful_revision_and_upsert_deployment(CreateDeploymentRevisionInput {
            guild_id: scope_id,
            entry: candidate.entry.clone(),
            files: candidate.files.clone(),
            bundle: candidate.bundle.clone(),
            source_map: candidate.source_map.clone(),
            status: DeploymentRevisionStatus::Success,
            deploy_source,
            actor_user_id: Some(identity.user_id),
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
            error!(target: "flora:custom-bots", bot_id = %bot_id, ?err, "failed to persist custom bot deployment");
            if let Some(previous) = previous {
                state
                    .custom_bot_gateway
                    .activate(bot_id, previous)
                    .await
                    .map_err(ApiError::internal)?;
            } else {
                state
                    .custom_bot_gateway
                    .deactivate(bot_id)
                    .await
                    .map_err(ApiError::internal)?;
            }
            return Err(ApiError::internal(err));
        }
    };
    Ok(ApiJson(Json(deployment.into())))
}
