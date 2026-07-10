use axum::{
    Router,
    routing::{delete, get, put},
};
use utoipa::OpenApi;

use crate::{
    handlers::{
        auth::{IdentityContext, ensure_guild_admin},
        custom_bots::{ensure_feature_enabled, ensure_scope_owner},
        error::ApiError,
    },
    services::custom_bots::CUSTOM_BOT_DEPLOYMENT_PREFIX,
    state::AppState,
};

mod delete_secret;
mod list;
mod upsert;

pub use delete_secret::*;
pub use list::*;
pub use upsert::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_secrets_handler,
        upsert_secret_handler,
        delete_secret_handler,
    ),
    components(schemas(
        SecretMetadataResponse,
        UpsertSecretRequest,
    )),
    tags((name = "Secrets", description = "Manage per-guild secrets"))
)]
pub struct SecretsApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{guild_id}", get(list_secrets_handler))
        .route("/{guild_id}/{name}", put(upsert_secret_handler))
        .route("/{guild_id}/{name}", delete(delete_secret_handler))
}

async fn ensure_secret_scope_access(
    state: &AppState,
    identity: &IdentityContext,
    scope_id: &str,
) -> Result<(), ApiError> {
    if scope_id.starts_with(CUSTOM_BOT_DEPLOYMENT_PREFIX) {
        ensure_feature_enabled(state, identity).await?;
        ensure_scope_owner(state, identity, scope_id).await?;
        Ok(())
    } else {
        ensure_guild_admin(state, identity, scope_id).await
    }
}

async fn refresh_secret_scope(state: &AppState, scope_id: &str) -> Result<(), ApiError> {
    if let Some(bot_id) = crate::services::custom_bots::custom_bot_id_from_deployment_id(scope_id) {
        if !state.custom_bot_gateway.is_running(bot_id) {
            return Ok(());
        }
        state
            .runtime
            .refresh_custom_bot_secrets(&bot_id.to_string(), scope_id)
            .await
            .map_err(ApiError::internal)
    } else {
        state
            .runtime
            .refresh_guild_secrets(scope_id)
            .await
            .map_err(ApiError::internal)
    }
}
