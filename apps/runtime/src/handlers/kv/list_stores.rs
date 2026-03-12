use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::kv::KvStore,
    state::AppState,
};
use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListStoresQuery {
    pub guild_id: String,
}

/// List all KV stores for a guild
#[utoipa::path(
    get,
    path = "/stores",
    params(
        ListStoresQuery
    ),
    summary = "List stores",
    description = "Returns all key-value stores for the specified guild.",
    responses(
        (status = 200, description = "List of stores", body = Vec<KvStore>),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not guild admin"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "KV"
)]
pub async fn list_stores_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListStoresQuery>,
) -> Result<ApiJson<Vec<KvStore>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &query.guild_id).await?;
    let stores = state.kv.list_stores(&query.guild_id).await?;
    Ok(ApiJson(Json(stores)))
}
