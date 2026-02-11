use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, get, put},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::{OpenApi, ToSchema};

use crate::{
    guild_bots::GuildBotBinding,
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

/// BYOB binding endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(get_guild_bot_binding_handler, upsert_guild_bot_binding_handler, delete_guild_bot_binding_handler),
    components(schemas(GuildBotBindingResponse, UpsertGuildBotBindingRequest, crate::handlers::error::ErrorResponse)),
    tags((name = "guild-bots", description = "Per-guild BYOB bot bindings"))
)]
pub struct GuildBotApi;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/{guild_id}", get(get_guild_bot_binding_handler))
        .route("/{guild_id}", put(upsert_guild_bot_binding_handler))
        .route("/{guild_id}", delete(delete_guild_bot_binding_handler))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GuildBotBindingResponse {
    pub guild_id: String,
    pub owner_user_id: String,
    pub bot_user_id: String,
    pub bot_username: String,
    pub application_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<GuildBotBinding> for GuildBotBindingResponse {
    fn from(value: GuildBotBinding) -> Self {
        Self {
            guild_id: value.guild_id,
            owner_user_id: value.owner_user_id,
            bot_user_id: value.bot_user_id,
            bot_username: value.bot_username,
            application_id: value.application_id,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertGuildBotBindingRequest {
    /// Discord bot token in `Bot <token>` format.
    pub bot_token: String,
}

/// Read BYOB binding metadata for a guild.
#[utoipa::path(
    get,
    path = "/{guild_id}",
    tag = "guild-bots",
    params(
        ("guild_id" = String, Path, description = "Discord guild id")
    ),
    responses(
        (status = 200, description = "Binding found", body = GuildBotBindingResponse),
        (status = 404, description = "Binding not found", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn get_guild_bot_binding_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<GuildBotBindingResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let Some(binding) = state
        .guild_bots
        .get_binding(&guild_id)
        .await
        .map_err(ApiError::internal)?
    else {
        return Err(ApiError::not_found("guild bot binding not found"));
    };

    Ok(ApiJson(Json(binding.into())))
}

/// Create or replace a guild BYOB binding.
#[utoipa::path(
    put,
    path = "/{guild_id}",
    tag = "guild-bots",
    params(
        ("guild_id" = String, Path, description = "Discord guild id")
    ),
    request_body = UpsertGuildBotBindingRequest,
    responses(
        (status = 200, description = "Binding stored", body = GuildBotBindingResponse),
        (status = 400, description = "Invalid input", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn upsert_guild_bot_binding_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpsertGuildBotBindingRequest>,
) -> Result<ApiJson<GuildBotBindingResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let binding = state
        .guild_bots
        .upsert_binding(&guild_id, &identity.user_id, &body.bot_token)
        .await
        .map_err(ApiError::internal)?;

    if let Err(err) = state.bot_gateway.sync_guild_binding(&guild_id).await {
        error!(target: "flora:api", guild_id, ?err, "failed to sync byob gateway after upsert");
        return Err(ApiError::internal(err));
    }

    Ok(ApiJson(Json(binding.into())))
}

/// Delete a guild BYOB binding.
#[utoipa::path(
    delete,
    path = "/{guild_id}",
    tag = "guild-bots",
    params(
        ("guild_id" = String, Path, description = "Discord guild id")
    ),
    responses(
        (status = 200, description = "Binding deleted"),
        (status = 404, description = "Binding not found", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn delete_guild_bot_binding_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<()>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let deleted = state
        .guild_bots
        .delete_binding(&guild_id)
        .await
        .map_err(ApiError::internal)?;

    state
        .bot_gateway
        .sync_guild_binding(&guild_id)
        .await
        .map_err(ApiError::internal)?;

    if !deleted {
        return Err(ApiError::not_found("guild bot binding not found"));
    }

    Ok(ApiJson(Json(())))
}
