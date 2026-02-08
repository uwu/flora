use axum::{Json, extract::State, http::HeaderMap};
use tracing::error;

use crate::{
    bundler::bundle_files,
    handlers::{
        auth::require_identity,
        deployments::{DeploymentRequest, DeploymentResponse},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/@me/deployment",
    request_body = DeploymentRequest,
    tag = "user",
    responses(
        (status = 200, description = "Deployment stored", body = DeploymentResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn upsert_user_deployment_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DeploymentRequest>,
) -> Result<ApiJson<DeploymentResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    let user_id = identity.user_id;

    let bundle_name = format!("user:{user_id}.bundle.js");
    let bundled = bundle_files(&bundle_name, &request.entry, &request.files)
        .map_err(|err| ApiError::bad_request(err.to_string()))?;

    let deployment = state
        .deployments
        .upsert_deployment(
            "user".to_string(),
            user_id.clone(),
            request.entry,
            request.files,
            bundled.code,
        )
        .await
        .map_err(|err| {
            error!(target: "flora:api", user_id, ?err, "failed to upsert user deployment");
            ApiError::internal(err)
        })?;

    state
        .runtime
        .deploy_script(deployment.clone())
        .await
        .map_err(|err| {
            error!(target: "flora:api", user_id, ?err, "failed to deploy user script");
            ApiError::internal(err)
        })?;

    Ok(ApiJson(Json(deployment.into())))
}

#[utoipa::path(
    get,
    path = "/@me/deployment",
    tag = "user",
    responses(
        (status = 200, description = "Deployment found", body = DeploymentResponse),
        (status = 404, description = "Deployment not found", body = crate::handlers::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn get_user_deployment_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<DeploymentResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    let user_id = identity.user_id;

    let deployment = state
        .deployments
        .get_deployment("user", &user_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", user_id, ?err, "failed to fetch user deployment");
            ApiError::internal(err)
        })?;

    let Some(deployment) = deployment else {
        return Err(ApiError::not_found("deployment not found"));
    };

    let files = deployment.files.clone();
    let response = DeploymentResponse::from(deployment).with_files(files);
    Ok(ApiJson(Json(response)))
}

#[utoipa::path(
    delete,
    path = "/@me/deployment",
    tag = "user",
    responses(
        (status = 204, description = "Deployment deleted"),
        (status = 404, description = "Deployment not found", body = crate::handlers::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn delete_user_deployment_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<()>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    let user_id = identity.user_id;

    let deployment = state
        .deployments
        .get_deployment("user", &user_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", user_id, ?err, "failed to fetch user deployment");
            ApiError::internal(err)
        })?;

    let Some(_) = deployment else {
        return Err(ApiError::not_found("deployment not found"));
    };

    state
        .deployments
        .delete_deployment("user", &user_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", user_id, ?err, "failed to delete user deployment");
            ApiError::internal(err)
        })?;

    Ok(ApiJson(Json(())))
}
