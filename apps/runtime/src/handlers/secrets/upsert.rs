use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertSecretRequest {
    pub value: String,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
}

#[utoipa::path(
    put,
    path = "/{guild_id}/{name}",
    tag = "secrets",
    params(
        ("guild_id" = String, Path, description = "Discord guild id"),
        ("name" = String, Path, description = "Secret name")
    ),
    request_body = UpsertSecretRequest,
    responses(
        (status = 200, description = "Secret stored", body = super::SecretMetadataResponse),
        (status = 401, description = "Unauthorized", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn upsert_secret_handler(
    Path((guild_id, name)): Path<(String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpsertSecretRequest>,
) -> Result<ApiJson<super::SecretMetadataResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let metadata = state
        .secrets
        .upsert_secret(&guild_id, &name, &body.value, body.allowed_hosts)
        .await
        .map_err(ApiError::internal)?;

    state
        .runtime
        .refresh_secrets(&guild_id)
        .await
        .map_err(ApiError::internal)?;

    let response: super::SecretMetadataResponse = metadata.into();
    Ok(ApiJson(Json(response)))
}
