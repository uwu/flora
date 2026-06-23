use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

use super::upsert::{actor_from_identity, restore_runtime_deployment};
use super::{DeploymentRevisionResponse, parse_deploy_source};
use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::deployments::{
        CreateDeploymentRevisionInput, Deployment, DeploymentRevisionStatus, DeploymentService,
        DeploymentSource,
    },
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/{guild_id}/rollback/{revision_id}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("revision_id" = String, Path, description = "Successful revision ID to rollback to")
    ),
    tag = "Deployments",
    summary = "Rollback deployment",
    description = "Deploys a previous successful revision and records a new rollback revision.",
    responses(
        (status = 200, description = "Rollback created", body = DeploymentRevisionResponse),
        (status = 404, description = "Revision not found", body = crate::handlers::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn rollback_deployment_handler(
    Path((guild_id, revision_id)): Path<(String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<DeploymentRevisionResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let revision_id =
        Uuid::parse_str(&revision_id).map_err(|_| ApiError::bad_request("Invalid revision ID"))?;

    let revision = state
        .deployments
        .get_guild_revision(&guild_id, revision_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to fetch rollback revision");
            ApiError::internal(err)
        })?;

    let Some(revision) = revision else {
        return Err(ApiError::not_found("Deployment revision not found"));
    };

    if !matches!(revision.status, DeploymentRevisionStatus::Success) {
        return Err(ApiError::bad_request(
            "Rollback target must be a successful revision",
        ));
    }

    let source = parse_deploy_source(&headers)?;
    let source = match source {
        DeploymentSource::Unknown => DeploymentSource::Api,
        _ => source,
    };
    let actor = actor_from_identity(&state, &identity, &guild_id).await;

    let previous_deployment = state
        .deployments
        .get_persisted_deployment(&guild_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to fetch active deployment before rollback");
            ApiError::internal(err)
        })?;

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

    let change_summary = DeploymentService::summarize_changes(revision.files.as_ref(), base_files);

    let now = Utc::now();
    let deployment = Deployment {
        guild_id: guild_id.clone(),
        entry: revision.entry.clone(),
        files: revision.files.clone(),
        source_map: revision.source_map.clone(),
        bundle: revision.bundle.clone(),
        created_at: now,
        updated_at: now,
    };

    if let Err(err) = state.runtime.deploy_guild_script(deployment.clone()).await {
        let error_message = Some(err.to_string());
        state
            .deployments
            .create_revision(CreateDeploymentRevisionInput {
                guild_id: guild_id.clone(),
                entry: deployment.entry.clone(),
                files: deployment.files.clone(),
                bundle: deployment.bundle.clone(),
                source_map: deployment.source_map.clone(),
                status: DeploymentRevisionStatus::Failed,
                deploy_source: source,
                actor_user_id: actor.user_id,
                actor_username: actor.username,
                actor_type: actor.actor_type,
                error_message,
                build_id: None,
                base_revision_id,
                change_summary,
            })
            .await
            .map_err(|err| {
                error!(target: "flora:api", guild_id, ?err, "failed to create failed rollback revision");
                ApiError::internal(err)
            })?;

        error!(target: "flora:api", guild_id, ?err, "failed to deploy rollback revision");
        return Err(ApiError::bad_request(format!("rollback failed: {err}")));
    }

    let (new_revision, _) = match state
        .deployments
        .create_successful_revision_and_upsert_deployment(CreateDeploymentRevisionInput {
            guild_id: guild_id.clone(),
            entry: deployment.entry.clone(),
            files: deployment.files.clone(),
            bundle: deployment.bundle.clone(),
            source_map: deployment.source_map.clone(),
            status: DeploymentRevisionStatus::Success,
            deploy_source: source,
            actor_user_id: actor.user_id,
            actor_username: actor.username,
            actor_type: actor.actor_type,
            error_message: None,
            build_id: None,
            base_revision_id,
            change_summary,
        })
        .await
    {
        Ok(result) => result,
        Err(err) => {
            error!(target: "flora:api", guild_id, ?err, "failed to persist successful rollback; restoring previous runtime");
            restore_runtime_deployment(
                &state,
                &guild_id,
                previous_deployment,
                "rollback persistence failure",
            )
            .await?;
            return Err(ApiError::internal(err));
        }
    };

    Ok(ApiJson(Json(DeploymentRevisionResponse::from_revision(
        new_revision,
        false,
    ))))
}
