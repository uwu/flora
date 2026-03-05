use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::kv::{RawKvKeyInfo, RawKvListKeysResult},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListKeysParams {
    pub guild_id: String,
    pub store_name: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListKeysQuery {
    pub prefix: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListKeysResponse {
    pub keys: Vec<RawKvKeyInfo>,
    pub list_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[utoipa::path(
    get,
    path = "/{guild_id}/{store_name}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("store_name" = String, Path, description = "Store name"),
        ListKeysQuery
    ),
    responses(
        (status = 200, description = "List of keys", body = ListKeysResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not guild admin"),
        (status = 404, description = "Store not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn list_keys_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(params): Path<ListKeysParams>,
    Query(query): Query<ListKeysQuery>,
) -> Result<ApiJson<RawKvListKeysResult>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &params.guild_id).await?;
    let result = state
        .kv
        .list_keys(
            &params.guild_id,
            &params.store_name,
            query.prefix.as_deref(),
            query.limit,
            query.cursor.as_deref(),
        )
        .await?;
    Ok(ApiJson(Json(result)))
}
