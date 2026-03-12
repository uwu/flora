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
    state::AppState,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteSecretResponse {
    pub deleted: bool,
}

#[utoipa::path(
    delete,
    path = "/{guild_id}/{name}",
    tag = "Secrets",
    summary = "Delete a secret",
    description = "Deletes a secret and refreshes the runtime to remove it.",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("name" = String, Path, description = "Secret name")
    ),
    responses(
        (status = 200, description = "Secret deleted", body = DeleteSecretResponse),
        (status = 401, description = "Unauthorized", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn delete_secret_handler(
    Path((guild_id, name)): Path<(String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<DeleteSecretResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    state
        .secrets
        .delete_secret(&guild_id, &name)
        .await
        .map_err(ApiError::internal)?;

    state
        .runtime
        .refresh_guild_secrets(&guild_id)
        .await
        .map_err(ApiError::internal)?;

    Ok(ApiJson(Json(DeleteSecretResponse { deleted: true })))
}
