use axum::extract::{Path, State};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{handlers::error::ApiError, state::AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteStoreParams {
    pub guild_id: String,
    pub store_name: String,
}

/// Delete a KV store and all its data
#[utoipa::path(
    delete,
    path = "/api/kv/stores/{guild_id}/{store_name}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("store_name" = String, Path, description = "Store name"),
    ),
    responses(
        (status = 200, description = "Store deleted successfully"),
        (status = 404, description = "Store not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn delete_store_handler(
    State(state): State<AppState>,
    Path(params): Path<DeleteStoreParams>,
) -> Result<(), ApiError> {
    state.kv.delete_store(&params.guild_id, &params.store_name).await?;
    Ok(())
}
