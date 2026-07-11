use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, get, put},
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::{orchestrator::FeatureAuthorizationRequest, server_custom_bots::ServerCustomBot},
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(get_server_custom_bot, upsert_server_custom_bot, delete_server_custom_bot),
    components(schemas(ServerCustomBotResponse, UpsertServerCustomBotRequest)),
    tags((name = "server-custom-bots", description = "Guild-owned Discord identities"))
)]
pub struct ServerCustomBotsApi;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/{guild_id}", get(get_server_custom_bot))
        .route("/{guild_id}", put(upsert_server_custom_bot))
        .route("/{guild_id}", delete(delete_server_custom_bot))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertServerCustomBotRequest {
    /// Discord bot token. It is validated, encrypted, and never returned.
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ServerCustomBotResponse {
    pub guild_id: String,
    pub bot_user_id: String,
    pub bot_username: String,
    pub application_id: String,
    pub running: bool,
    pub created_at: String,
    pub updated_at: String,
}

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    guild_id: &str,
) -> Result<crate::handlers::auth::IdentityContext, ApiError> {
    let identity = require_identity(state, headers).await?;
    ensure_guild_admin(state, &identity, guild_id).await?;
    let allowed = state
        .runtime
        .authorize_feature(FeatureAuthorizationRequest {
            feature: "server_custom_bots".to_string(),
            user_id: identity.user_id.clone(),
            metadata: Some(serde_json::json!({ "guildId": guild_id })),
        })
        .await
        .unwrap_or(false);
    if !allowed {
        return Err(ApiError::forbidden(
            "server custom bots are not enabled for this guild",
        ));
    }
    Ok(identity)
}

fn response(state: &AppState, bot: ServerCustomBot) -> ServerCustomBotResponse {
    ServerCustomBotResponse {
        running: state.server_custom_bot_gateway.is_running(&bot.guild_id),
        guild_id: bot.guild_id,
        bot_user_id: bot.bot_user_id,
        bot_username: bot.bot_username,
        application_id: bot.application_id,
        created_at: bot.created_at.to_rfc3339(),
        updated_at: bot.updated_at.to_rfc3339(),
    }
}

#[utoipa::path(get, path = "/{guild_id}", tag = "server-custom-bots", params(("guild_id" = String, Path)), responses((status = 200, body = Option<ServerCustomBotResponse>)))]
pub async fn get_server_custom_bot(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<Option<ServerCustomBotResponse>>, ApiError> {
    authorize(&state, &headers, &guild_id).await?;
    let bot = state
        .server_custom_bots
        .get(&guild_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(ApiJson(Json(bot.map(|bot| response(&state, bot)))))
}

#[utoipa::path(put, path = "/{guild_id}", tag = "server-custom-bots", params(("guild_id" = String, Path)), request_body = UpsertServerCustomBotRequest, responses((status = 200, body = ServerCustomBotResponse)))]
pub async fn upsert_server_custom_bot(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpsertServerCustomBotRequest>,
) -> Result<ApiJson<ServerCustomBotResponse>, ApiError> {
    let identity = authorize(&state, &headers, &guild_id).await?;
    let previous = state
        .server_custom_bots
        .get_with_token(&guild_id)
        .await
        .map_err(ApiError::internal)?;
    let validated = state
        .server_custom_bots
        .validate_token(&guild_id, &request.token)
        .await
        .map_err(|err| ApiError::bad_request(err.to_string()))?;
    let bot = state
        .server_custom_bots
        .upsert(&guild_id, &identity.user_id, validated)
        .await
        .map_err(ApiError::internal)?;
    if let Err(err) = state.server_custom_bot_gateway.activate(&guild_id).await {
        if previous.is_some() {
            state
                .server_custom_bots
                .restore(previous)
                .await
                .map_err(ApiError::internal)?;
        } else {
            state
                .server_custom_bots
                .delete(&guild_id)
                .await
                .map_err(ApiError::internal)?;
        }
        return Err(ApiError::internal(err));
    }
    Ok(ApiJson(Json(response(&state, bot))))
}

#[utoipa::path(delete, path = "/{guild_id}", tag = "server-custom-bots", params(("guild_id" = String, Path)), responses((status = 200)))]
pub async fn delete_server_custom_bot(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<()>, ApiError> {
    authorize(&state, &headers, &guild_id).await?;
    state
        .server_custom_bot_gateway
        .deactivate(&guild_id)
        .await
        .map_err(ApiError::internal)?;
    state
        .server_custom_bots
        .delete(&guild_id)
        .await
        .map_err(ApiError::internal)?;
    Ok(ApiJson(Json(())))
}
