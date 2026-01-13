use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
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
pub struct ExportGuildParams {
    pub guild_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExportGuildResponse {
    pub backup_id: String,
}

/// Export all KV stores for a guild
///
/// Creates a backup of all stores using the sled database export format.
/// Returns a backup ID for later retrieval.
#[utoipa::path(
    post,
    path = "/api/kv/export/{guild_id}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
    ),
    responses(
        (status = 200, description = "Export created successfully", body = ExportGuildResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not guild admin"),
        (status = 404, description = "No stores found for guild"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn export_guild_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(params): Path<ExportGuildParams>,
) -> Result<ApiJson<ExportGuildResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &params.guild_id).await?;
    let backup_id = state.kv.export_guild(&params.guild_id).await?;
    Ok(ApiJson(Json(ExportGuildResponse { backup_id })))
}
