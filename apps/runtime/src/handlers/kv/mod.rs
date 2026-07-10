use crate::{
    handlers::{
        auth::{IdentityContext, ensure_guild_admin},
        custom_bots::{ensure_feature_enabled, ensure_scope_owner},
        error::ApiError,
    },
    services::{custom_bots::CUSTOM_BOT_DEPLOYMENT_PREFIX, kv::KvStore},
    state::AppState,
};
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use utoipa::OpenApi;

mod create_store;
mod delete_key;
mod delete_store;
mod export;
mod get;
mod list_keys;
mod list_stores;
mod set;

pub use create_store::*;
pub use delete_key::*;
pub use delete_store::*;
pub use export::*;
pub use get::*;
pub use list_keys::*;
pub use list_stores::*;
pub use set::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stores", post(create_store_handler))
        .route("/stores", get(list_stores_handler))
        .route(
            "/stores/{guild_id}/{store_name}",
            delete(delete_store_handler),
        )
        .route("/{guild_id}/{store_name}/{key}", get(get_value_handler))
        .route("/{guild_id}/{store_name}/{key}", put(set_value_handler))
        .route("/{guild_id}/{store_name}/{key}", delete(delete_key_handler))
        .route("/{guild_id}/{store_name}", get(list_keys_handler))
        .route("/export/{guild_id}", post(export_guild_handler))
}

async fn ensure_kv_scope_access(
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

#[derive(OpenApi)]
#[openapi(
    paths(
        create_store_handler,
        list_stores_handler,
        delete_store_handler,
        get_value_handler,
        set_value_handler,
        delete_key_handler,
        list_keys_handler,
        export_guild_handler,
    ),
    components(schemas(
        KvStore,
        CreateStoreRequest,
        CreateStoreResponse,
        ListStoresQuery,
        DeleteStoreParams,
        GetValueParams,
        SetValueParams,
        SetValueRequest,
        DeleteKeyParams,
        ListKeysParams,
        ListKeysQuery,
        ListKeysResponse,
        ExportGuildParams,
    )),
    tags(
        (name = "KV", description = "Key-value store management endpoints")
    )
)]
pub struct KvApi;
