use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    kv::KvStore,
    state::AppState,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateStoreRequest {
    pub guild_id: String,
    pub store_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateStoreResponse {
    pub store: KvStore,
}

/// Create a new KV store for a guild
#[utoipa::path(
    post,
    path = "/stores",
    request_body = CreateStoreRequest,
    responses(
        (status = 200, description = "Store created successfully", body = CreateStoreResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not guild admin"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn create_store_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateStoreRequest>,
) -> Result<ApiJson<CreateStoreResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &req.guild_id).await?;
    let store = state.kv.create_store(req.guild_id, req.store_name).await?;
    Ok(ApiJson(Json(CreateStoreResponse { store })))
}
