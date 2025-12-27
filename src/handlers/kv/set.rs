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
pub struct SetValueParams {
    pub guild_id: String,
    pub store_name: String,
    pub key: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetValueRequest {
    pub value: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetValueResponse {
    pub success: bool,
}

/// Set a value in a KV store
///
/// The value size is limited to 1MB.
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
        (status = 404, description = "Store not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn set_value_handler(
    State(state): State<AppState>,
    Path(params): Path<SetValueParams>,
    Json(req): Json<SetValueRequest>,
) -> Result<ApiJson<SetValueResponse>, ApiError> {
    state.kv.set(&params.guild_id, &params.store_name, &params.key, &req.value).await?;
    Ok(ApiJson(Json(SetValueResponse { success: true })))
}
