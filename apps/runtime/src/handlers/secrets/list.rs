use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::secrets::SecretMetadata,
    state::AppState,
};

/// Metadata response; values are never returned.
#[derive(Debug, Serialize, ToSchema)]
pub struct SecretMetadataResponse {
    pub name: String,
    pub allowed_hosts: Vec<String>,
}

impl From<SecretMetadata> for SecretMetadataResponse {
    fn from(value: SecretMetadata) -> Self {
        Self {
            name: value.name,
            allowed_hosts: value.allowed_hosts,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{guild_id}",
    tag = "secrets",
    params(
        ("guild_id" = String, Path, description = "Discord guild id")
    ),
    responses(
        (status = 200, description = "List of secret names", body = [SecretMetadataResponse]),
        (status = 401, description = "Unauthorized", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn list_secrets_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<Vec<SecretMetadataResponse>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let secrets = state
        .secrets
        .list_metadata(&guild_id)
        .await
        .map_err(ApiError::internal)?;

    let response: Vec<_> = secrets
        .into_iter()
        .map(SecretMetadataResponse::from)
        .collect();

    Ok(ApiJson(Json(response)))
}
