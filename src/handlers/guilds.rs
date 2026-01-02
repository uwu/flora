use axum::{Json, Router, extract::State, http::HeaderMap, routing::get};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::{
    handlers::{
        auth::{has_admin_permissions, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

/// Guild listing endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(list_guilds_handler),
    components(schemas(GuildResponse, crate::handlers::error::ErrorResponse)),
    tags((name = "guilds", description = "Guilds the user can manage"))
)]
pub struct GuildApi;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_guilds_handler))
}

/// Guild info exposed by the API.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuildResponse {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: u64,
}

/// List guilds where the user is an admin and the bot is present.
#[utoipa::path(
    get,
    path = "/",
    tag = "guilds",
    responses(
        (status = 200, description = "Guilds available for deployment", body = [GuildResponse]),
        (status = 401, description = "Not authenticated", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn list_guilds_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<Vec<GuildResponse>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    let access_token = identity
        .access_token
        .ok_or_else(|| ApiError::forbidden("user session required for guild listing"))?;
    let guilds =
        state.auth.fetch_user_guilds(&access_token).await.map_err(|err| ApiError::internal(err))?;

    let mut allowed = Vec::new();
    for guild in guilds {
        let perms = guild
            .permissions_new
            .as_deref()
            .or(guild.permissions.as_deref())
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or_default();

        if !has_admin_permissions(perms) {
            continue;
        }

        let guild_id_u64: u64 = match guild.id.parse() {
            Ok(id) => id,
            Err(err) => {
                tracing::warn!(guild_id = %guild.id, "failed to parse guild ID: {}", err);
                continue;
            }
        };

        match state.http.get_guild(guild_id_u64.into()).await {
            Ok(_) => {
                allowed.push(GuildResponse {
                    id: guild.id,
                    name: guild.name,
                    icon: guild.icon,
                    permissions: perms,
                });
            }
            Err(err) => {
                tracing::debug!(guild_id = %guild.id, guild_name = %guild.name, "bot not in guild or failed to fetch: {}", err);
            }
        }
    }

    Ok(ApiJson(Json(allowed)))
}
