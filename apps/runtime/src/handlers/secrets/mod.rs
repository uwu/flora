use axum::{
    Router,
    routing::{delete, get, put},
};
use utoipa::OpenApi;

use crate::state::AppState;

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
    tags((name = "secrets", description = "Manage per-guild secrets"))
)]
pub struct SecretsApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{guild_id}", get(list_secrets_handler))
        .route("/{guild_id}/{name}", put(upsert_secret_handler))
        .route("/{guild_id}/{name}", delete(delete_secret_handler))
}
