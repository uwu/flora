use axum::extract::{Path, State};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{handlers::error::ApiError, state::AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteKeyParams {
    pub guild_id: String,
    pub store_name: String,
    pub key: String,
}

/// Delete a key from a KV store
#[utoipa::path(
    delete,
    path = "/api/kv/{guild_id}/{store_name}/{key}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("store_name" = String, Path, description = "Store name"),
        ("key" = String, Path, description = "Key to delete"),
    ),
    responses(
        (status = 200, description = "Key deleted successfully"),
        (status = 404, description = "Store not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn delete_key_handler(
    State(state): State<AppState>,
    Path(params): Path<DeleteKeyParams>,
) -> Result<(), ApiError> {
    state.kv.delete(&params.guild_id, &params.store_name, &params.key).await?;
    Ok(())
}
