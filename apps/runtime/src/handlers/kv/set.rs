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
pub struct SetValueParams {
    pub guild_id: String,
    pub store_name: String,
    pub key: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetValueRequest {
    pub value: String,
    pub expiration: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetValueResponse {
    pub success: bool,
}

#[utoipa::path(
    put,
    path = "/api/kv/{guild_id}/{store_name}/{key}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("store_name" = String, Path, description = "Store name"),
        ("key" = String, Path, description = "Key to set"),
    ),
    request_body = SetValueRequest,
    responses(
        (status = 200, description = "Value set successfully", body = SetValueResponse),
        (status = 400, description = "Value exceeds 1MB limit"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not guild admin"),
        (status = 404, description = "Store not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn set_value_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(params): Path<SetValueParams>,
    Json(req): Json<SetValueRequest>,
) -> Result<ApiJson<SetValueResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &params.guild_id).await?;
    state
        .kv
        .set(
            &params.guild_id,
            &params.store_name,
            &params.key,
            &req.value,
            req.expiration,
            req.metadata,
        )
        .await?;
    Ok(ApiJson(Json(SetValueResponse { success: true })))
}
