use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use super::DeploymentRevisionResponse;
use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    services::deployments::DeploymentRevisionCursor,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct DeploymentHistoryQuery {
    #[serde(default = "default_history_limit")]
    limit: i64,
    cursor_deployed_at: Option<String>,
    cursor_id: Option<String>,
    #[serde(default)]
    include_bundle: bool,
}

fn default_history_limit() -> i64 {
    25
}

#[utoipa::path(
    get,
    path = "/{guild_id}/history",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        ("limit" = Option<i64>, Query, description = "Page size, max 100"),
        ("cursor_deployed_at" = Option<String>, Query, description = "RFC3339 deployed_at cursor"),
        ("cursor_id" = Option<String>, Query, description = "Revision ID cursor"),
        ("include_bundle" = Option<bool>, Query, description = "Include bundled output in response")
    ),
    tag = "Deployments",
    summary = "List deployment history",
    description = "Returns deployment revisions for a guild in reverse chronological order.",
    responses(
        (status = 200, description = "Revision history", body = [DeploymentRevisionResponse]),
        (status = 400, description = "Invalid cursor", body = crate::handlers::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn list_deployment_history_handler(
    Path(guild_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DeploymentHistoryQuery>,
) -> Result<ApiJson<Vec<DeploymentRevisionResponse>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let limit = query.limit.clamp(1, 100);
    let cursor = parse_cursor(query.cursor_deployed_at, query.cursor_id)?;

    let revisions = state
        .deployments
        .list_guild_revisions(&guild_id, limit, cursor)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to list deployment revisions");
            ApiError::internal(err)
        })?;

    let mut response = Vec::with_capacity(revisions.len());
    for revision in revisions {
        response.push(DeploymentRevisionResponse::from_revision(
            revision,
            query.include_bundle,
        ));
    }

    Ok(ApiJson(Json(response)))
}

fn parse_cursor(
    cursor_deployed_at: Option<String>,
    cursor_id: Option<String>,
) -> Result<Option<DeploymentRevisionCursor>, ApiError> {
    match (cursor_deployed_at, cursor_id) {
        (None, None) => Ok(None),
        (Some(_), None) | (None, Some(_)) => Err(ApiError::bad_request(
            "`cursor_deployed_at` and `cursor_id` must be provided together",
        )),
        (Some(cursor_deployed_at), Some(cursor_id)) => {
            let deployed_at = DateTime::parse_from_rfc3339(&cursor_deployed_at)
                .map_err(|_| ApiError::bad_request("`cursor_deployed_at` must be RFC3339"))?
                .with_timezone(&Utc);
            let id = Uuid::parse_str(&cursor_id)
                .map_err(|_| ApiError::bad_request("`cursor_id` must be a UUID"))?;
            Ok(Some(DeploymentRevisionCursor { deployed_at, id }))
        }
    }
}
