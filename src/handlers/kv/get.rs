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
pub struct GetValueParams {
    pub guild_id: String,
    pub store_name: String,
    pub key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetValueResponse {
    pub value: Option<String>,
}

/// Get a value from a KV store
#[utoipa::path(
    get,
    path = "/api/kv/{guild_id}/{store_name}/{key}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("store_name" = String, Path, description = "Store name"),
        ("key" = String, Path, description = "Key to retrieve"),
    ),
    responses(
        (status = 200, description = "Value retrieved", body = GetValueResponse),
        (status = 404, description = "Store or key not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "kv"
)]
pub async fn get_value_handler(
    State(state): State<AppState>,
    Path(params): Path<GetValueParams>,
) -> Result<ApiJson<GetValueResponse>, ApiError> {
    let value = state.kv.get(&params.guild_id, &params.store_name, &params.key).await?;
    Ok(ApiJson(Json(GetValueResponse { value })))
}
