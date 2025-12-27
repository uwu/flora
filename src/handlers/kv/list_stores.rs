use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    handlers::{error::ApiError, response::ApiJson},
    kv::KvStore,
    state::AppState,
};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct ListStoresQuery {
    pub guild_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListStoresResponse {
    pub stores: Vec<KvStore>,
}

/// List all KV stores for a guild
#[utoipa::path(
    get,
    path = "/api/kv/stores",
    params(
        ListStoresQuery
    ),
    responses(
        (status = 200, description = "List of stores", body = Vec<KvStore>),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn list_stores_handler(
    State(state): State<AppState>,
    Query(query): Query<ListStoresQuery>,
) -> Result<ApiJson<Vec<KvStore>>, ApiError> {
    let stores = state.kv.list_stores(&query.guild_id).await?;
    Ok(ApiJson(Json(stores)))
}
