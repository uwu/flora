//! Log streaming endpoint using Server-Sent Events (SSE).

use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, Query},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
};
use serde::Deserialize;
use tokio_stream::StreamExt as _;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::log_sink::{self, LogEntry};

#[derive(OpenApi)]
#[openapi(
    paths(get_logs, get_guild_logs),
    components(schemas(LogEntry, LogsQuery))
)]
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

/// Get recent logs.
#[utoipa::path(
    get,
    path = "",
    params(LogsQuery),
    responses(
        (status = 200, description = "Recent log entries", body = Vec<LogEntry>)
    ),
    tag = "logs"
)]
pub async fn get_logs(Query(query): Query<LogsQuery>) -> impl IntoResponse {
    let limit = query.limit.min(1000);
    let logs = log_sink::log_sink().recent(limit);
    Json(logs)
}

/// Stream logs via Server-Sent Events.
///
/// Note: This endpoint is not documented in OpenAPI due to SSE response type limitations.
pub async fn stream_logs() -> impl IntoResponse {
    let receiver = log_sink::log_sink().subscribe();

    let stream = tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(|result| {
        match result {
            Ok(entry) => {
                let json = serde_json::to_string(&entry).ok()?;
                Some(Ok::<_, Infallible>(Event::default().data(json)))
            }
            Err(_) => None, // Skip lagged messages
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Get recent logs for a specific guild.
#[utoipa::path(
    get,
    path = "/{guild_id}",
    params(
        ("guild_id" = String, Path, description = "Discord guild ID"),
        LogsQuery
    ),
    responses(
        (status = 200, description = "Recent log entries for guild", body = Vec<LogEntry>)
    ),
    tag = "logs"
)]
pub async fn get_guild_logs(
    Path(guild_id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.min(1000);
    let logs = log_sink::log_sink().recent_for_guild(&guild_id, limit);
    Json(logs)
}

/// Stream logs for a specific guild via Server-Sent Events.
///
/// Note: This endpoint is not documented in OpenAPI due to SSE response type limitations.
pub async fn stream_guild_logs(Path(guild_id): Path<String>) -> impl IntoResponse {
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

    Sse::new(stream).keep_alive(KeepAlive::default())
}
