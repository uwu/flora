use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::{OpenApi, ToSchema};

use crate::{
    handlers::{
        auth::{ensure_guild_admin, require_identity},
        error::ApiError,
        response::ApiJson,
    },
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(create_build_handler, get_build_handler),
    components(schemas(CreateBuildResponse, BuildStatusResponse)),
    tags((name = "Builds", description = "Server-side build pipeline"))
)]
pub struct BuildApi;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_build_handler))
        .route("/{build_id}", get(get_build_handler))
        .route("/{build_id}/logs", get(stream_build_logs_handler))
}

/// Response returned when a build is created.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateBuildResponse {
    /// Build identifier.
    pub build_id: String,
    /// Current build status string.
    pub status: String,
}

/// Build artifact output paths.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BuildArtifactResponse {
    /// Bundled JavaScript output.
    pub bundle: String,
    /// Source map for the bundle.
    pub source_map: String,
}

/// Current build status and output.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BuildStatusResponse {
    /// Build identifier.
    pub build_id: String,
    /// Guild ID the build targets.
    pub guild_id: String,
    /// Entry file path used for the build.
    pub entry: String,
    /// Current build status string.
    pub status: String,
    /// Build logs, newest last.
    pub logs: Vec<String>,
    /// Build start time in RFC3339 (UTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// Build completion time in RFC3339 (UTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    /// Generated build artifacts, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<BuildArtifactResponse>,
    /// Error detail when the build fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[utoipa::path(
    post,
    path = "/",
    tag = "Builds",
    summary = "Create a build",
    description = "Queues a server-side build for the provided project archive.",
    responses(
        (status = 200, description = "Build queued", body = CreateBuildResponse),
        (status = 400, description = "Invalid request", body = crate::handlers::error::ErrorResponse),
        (status = 401, description = "Not authenticated", body = crate::handlers::error::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn create_build_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<ApiJson<CreateBuildResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;

    let mut guild_id: Option<String> = None;
    let mut entry: Option<String> = None;
    let mut project_zip: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::bad_request(format!("invalid multipart: {err}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "guild_id" => {
                guild_id =
                    Some(field.text().await.map_err(|err| {
                        ApiError::bad_request(format!("invalid guild_id: {err}"))
                    })?);
            }
            "entry" => {
                entry = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| ApiError::bad_request(format!("invalid entry: {err}")))?,
                );
            }
            "project_zip" => {
                project_zip = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|err| {
                            ApiError::bad_request(format!("invalid project_zip: {err}"))
                        })?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let guild_id = guild_id.ok_or_else(|| ApiError::bad_request("`guild_id` is required"))?;
    let entry = entry.ok_or_else(|| ApiError::bad_request("`entry` is required"))?;
    let project_zip =
        project_zip.ok_or_else(|| ApiError::bad_request("`project_zip` is required"))?;

    ensure_guild_admin(&state, &identity, &guild_id).await?;

    let result = state
        .build_service
        .create_build(&guild_id, &entry, project_zip)
        .await
        .map_err(|err| {
            error!(target: "flora:api", guild_id, ?err, "failed to create build");
            ApiError::internal(err)
        })?;

    Ok(ApiJson(Json(CreateBuildResponse {
        build_id: result.build_id,
        status: result.status,
    })))
}

#[utoipa::path(
    get,
    path = "/{build_id}",
    tag = "Builds",
    summary = "Get build status",
    description = "Returns the current status, logs, and artifacts for a build.",
    params(
        ("build_id" = String, Path, description = "Build ID")
    ),
    responses(
        (status = 200, description = "Build status", body = BuildStatusResponse),
        (status = 404, description = "Build not found", body = crate::handlers::error::ErrorResponse),
        (status = 401, description = "Not authenticated", body = crate::handlers::error::ErrorResponse)
    )
)]
pub async fn get_build_handler(
    Path(build_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<ApiJson<BuildStatusResponse>, ApiError> {
    let identity = require_identity(&state, &headers).await?;

    let build = state
        .build_service
        .get_build(&build_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", build_id, ?err, "failed to get build");
            ApiError::internal(err)
        })?;
    let build = build.ok_or_else(|| ApiError::not_found(format!("build {build_id} not found")))?;

    ensure_guild_admin(&state, &identity, &build.guild_id).await?;

    Ok(ApiJson(Json(BuildStatusResponse {
        build_id: build.build_id,
        guild_id: build.guild_id,
        entry: build.entry,
        status: build.status,
        logs: build.logs,
        started_at: build.started_at,
        finished_at: build.finished_at,
        artifact: build.artifact.map(|artifact| BuildArtifactResponse {
            bundle: artifact.bundle,
            source_map: artifact.source_map,
        }),
        error: build.error,
    })))
}

/// Stream build logs via SSE.
///
/// Note: Not documented in OpenAPI due to SSE response type limitations.
pub async fn stream_build_logs_handler(
    Path(build_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let identity = require_identity(&state, &headers).await?;

    let build = state
        .build_service
        .get_build(&build_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", build_id, ?err, "failed to get build");
            ApiError::internal(err)
        })?;
    let build = build.ok_or_else(|| ApiError::not_found(format!("build {build_id} not found")))?;

    ensure_guild_admin(&state, &identity, &build.guild_id).await?;

    let response = state
        .build_service
        .stream_logs(&build_id)
        .await
        .map_err(|err| {
            error!(target: "flora:api", build_id, ?err, "failed to stream build logs");
            ApiError::internal(err)
        })?;

    let stream = response.bytes_stream();

    Ok(Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap())
}
