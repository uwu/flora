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
    tags((name = "Server Custom Bots", description = "Guild-owned Discord identities for existing flora deployments"))
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
        return Err(ApiError::feature_disabled(
            "Server Custom Bots are not enabled for this server. Ask a flora operator to enable the feature before trying again",
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

#[utoipa::path(
    get,
    path = "/{guild_id}",
    tag = "Server Custom Bots",
    summary = "Get a Server Custom Bot",
    description = "Returns the Server Custom Bot configured for a guild, or null when the guild uses @Flora. The caller must manage the guild and the feature must be enabled by the orchestrator.",
    params(("guild_id" = String, Path, description = "Guild ID")),
    responses((status = 200, description = "Configured Server Custom Bot, or null when none is configured", body = Option<ServerCustomBotResponse>))
)]
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

#[utoipa::path(
    put,
    path = "/{guild_id}",
    tag = "Server Custom Bots",
    summary = "Configure a Server Custom Bot",
    description = "Validates that the supplied Discord bot token can access the guild, encrypts the token at rest, and routes the guild's existing flora deployment through that identity. Repeating the request replaces the token and gateway while preserving the guild deployment.",
    params(("guild_id" = String, Path, description = "Guild ID")),
    request_body = UpsertServerCustomBotRequest,
    responses((status = 200, description = "Configured Server Custom Bot", body = ServerCustomBotResponse))
)]
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

#[utoipa::path(
    delete,
    path = "/{guild_id}",
    tag = "Server Custom Bots",
    summary = "Remove a Server Custom Bot",
    description = "Stops the guild-owned gateway, deletes its encrypted token metadata, and returns the guild's existing deployment to @Flora. The caller must manage the guild.",
    params(("guild_id" = String, Path, description = "Guild ID")),
    responses((status = 200, description = "Server Custom Bot removed and @Flora restored"))
)]
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
