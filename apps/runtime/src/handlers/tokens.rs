use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, post},
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::{
    handlers::{auth::require_session, error::ApiError, response::ApiJson},
    services::tokens::UserToken,
    state::AppState,
};

/// Token management endpoints.
#[derive(OpenApi)]
#[openapi(
    paths(create_token_handler, list_tokens_handler, delete_token_handler),
    components(schemas(CreateTokenRequest, CreateTokenResponse, TokenResponse, crate::handlers::error::ErrorResponse)),
    tags((name = "Tokens", description = "User API tokens"))
)]
pub struct TokenApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_token_handler))
        .route("/", axum::routing::get(list_tokens_handler))
        .route("/{token_id}", delete(delete_token_handler))
}

/// Create-token payload.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTokenRequest {
    pub label: Option<String>,
}

/// Token creation response (includes plaintext token).
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateTokenResponse {
    pub token: String,
}

/// Token metadata returned for list endpoints.
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    /// Token identifier.
    pub token_id: String,
    /// Optional user-facing label for the token.
    pub label: Option<String>,
    /// Token creation time in RFC3339 (UTC).
    pub created_at: String,
    /// Last usage time in RFC3339 (UTC), if available.
    pub last_used_at: Option<String>,
}

impl From<UserToken> for TokenResponse {
    fn from(value: UserToken) -> Self {
        Self {
            token_id: value.token_id,
            label: value.label,
            created_at: value.created_at.to_rfc3339(),
            last_used_at: value.last_used_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// Mint a new API token for the authenticated user.
#[utoipa::path(
    post,
    path = "/",
    tag = "Tokens",
    summary = "Create a token",
    description = "Creates a new API token for the authenticated user. The plaintext token is returned only once.",
    request_body = CreateTokenRequest,
    responses(
        (status = 200, description = "Token created", body = CreateTokenResponse),
        (status = 401, description = "Unauthorized", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn create_token_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTokenRequest>,
) -> Result<ApiJson<CreateTokenResponse>, ApiError> {
    let session = require_session(&state.auth, &headers).await?;
    let token = state
        .tokens
        .create_token(&session.user.id, body.label)
        .await
        .map_err(ApiError::internal)?;

    Ok(ApiJson(Json(CreateTokenResponse { token })))
}

/// List tokens for the authenticated user.
#[utoipa::path(
    get,
    path = "/",
    tag = "Tokens",
    summary = "List tokens",
    description = "Returns metadata for all API tokens owned by the authenticated user.",
    responses(
        (status = 200, description = "Tokens listed", body = [TokenResponse]),
        (status = 401, description = "Unauthorized", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn list_tokens_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<Vec<TokenResponse>>, ApiError> {
    let session = require_session(&state.auth, &headers).await?;
    let tokens = state
        .tokens
        .list_tokens(&session.user.id)
        .await
        .map_err(ApiError::internal)?;
    Ok(ApiJson(Json(
        tokens.into_iter().map(TokenResponse::from).collect(),
    )))
}

/// Delete a token by id.
#[utoipa::path(
    delete,
    path = "/{token_id}",
    tag = "Tokens",
    summary = "Delete a token",
    description = "Deletes the specified token. Requests using this token stop authenticating immediately.",
    params(
        ("token_id" = String, Path, description = "Token identifier")
    ),
    responses(
        (status = 204, description = "Token deleted"),
        (status = 401, description = "Unauthorized", body = crate::handlers::error::ErrorResponse),
        (status = 404, description = "Token not found", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn delete_token_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token_id): Path<String>,
) -> Result<ApiJson<()>, ApiError> {
    let session = require_session(&state.auth, &headers).await?;
    let deleted = state
        .tokens
        .delete_token(&session.user.id, &token_id)
        .await
        .map_err(ApiError::internal)?;
    if deleted {
        Ok(ApiJson(Json(())))
    } else {
        Err(ApiError::not_found("Token not found"))
    }
}
