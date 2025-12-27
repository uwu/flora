use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    handlers::{error::ApiError, response::ApiJson},
    state::AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExportGuildParams {
    pub guild_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExportGuildResponse {
    pub backup_path: String,
}

/// Export all KV stores for a guild
///
/// Creates a backup of all stores using the sled database export format.
/// Returns the path to the backup directory.
#[utoipa::path(
    post,
    path = "/api/kv/export/{guild_id}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
    ),
    responses(
        (status = 200, description = "Export created successfully", body = ExportGuildResponse),
        (status = 404, description = "No stores found for guild"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn export_guild_handler(
    State(state): State<AppState>,
    Path(params): Path<ExportGuildParams>,
) -> Result<ApiJson<ExportGuildResponse>, ApiError> {
    let backup_path = state.kv.export_guild(&params.guild_id).await?;
    Ok(ApiJson(Json(ExportGuildResponse {
        backup_path: backup_path.to_string_lossy().to_string(),
    })))
}
