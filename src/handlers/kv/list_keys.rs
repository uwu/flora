use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    handlers::{error::ApiError, response::ApiJson},
    state::AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListKeysParams {
    pub guild_id: String,
    pub store_name: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListKeysQuery {
    /// Optional prefix to filter keys
    pub prefix: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListKeysResponse {
    pub keys: Vec<String>,
}

/// List all keys in a KV store
///
/// WARNING: This endpoint is not paginated. It may be slow for stores with millions of keys.
/// Consider using a prefix filter to narrow results.
#[utoipa::path(
    get,
    path = "/api/kv/{guild_id}/{store_name}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("store_name" = String, Path, description = "Store name"),
        ListKeysQuery
    ),
    responses(
        (status = 200, description = "List of keys", body = ListKeysResponse),
        (status = 404, description = "Store not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn list_keys_handler(
    State(state): State<AppState>,
    Path(params): Path<ListKeysParams>,
    Query(query): Query<ListKeysQuery>,
) -> Result<ApiJson<ListKeysResponse>, ApiError> {
    let keys =
        state.kv.list_keys(&params.guild_id, &params.store_name, query.prefix.as_deref()).await?;
    Ok(ApiJson(Json(ListKeysResponse { keys })))
}
