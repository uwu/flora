//! Log streaming endpoint using Server-Sent Events (SSE).

use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
};
use serde::Deserialize;
use std::convert::Infallible;
use tokio_stream::StreamExt as _;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    log_sink::{self, LogEntry},
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(paths(get_guild_logs), components(schemas(LogEntry, LogsQuery)))]
pub struct LogsApi;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct LogsQuery {
    /// Maximum number of log entries to return (default 100, max 1000).
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    100
}

/// Get recent logs for a specific guild.
#[utoipa::path(
    get,
    path = "/{guild_id}",
    params(
        ("guild_id" = String, Path, description = "Guild ID"),
        LogsQuery
    ),
    summary = "List guild logs",
    description = "Returns recent log entries for a specific guild.",
    responses(
        (status = 200, description = "Recent log entries for guild", body = Vec<LogEntry>)
    ),
    tag = "Logs"
)]
pub async fn get_guild_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<ApiJson<Vec<LogEntry>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;
    let limit = query.limit.min(1000);
    let logs = log_sink::log_sink().recent_for_guild(&guild_id, limit);
    Ok(ApiJson(Json(logs)))
}

/// Stream logs for a specific guild via Server-Sent Events.
///
/// Note: This endpoint is not documented in OpenAPI due to SSE response type limitations.
pub async fn stream_guild_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<String>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let identity = require_identity(&state, &headers).await?;
    ensure_guild_admin(&state, &identity, &guild_id).await?;
    let receiver = log_sink::log_sink().subscribe();

    let stream = tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(move |result| {
        match result {
            Ok(entry) => {
                // Filter to only this guild
                if entry.guild_id.as_deref() == Some(guild_id.as_str()) {
                    let json = serde_json::to_string(&entry).ok()?;
                    Some(Ok::<_, Infallible>(Event::default().data(json)))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
