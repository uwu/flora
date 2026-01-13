/// Simple liveness probe.
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(health_check),
    tags((name = "health", description = "Health endpoints"))
)]
pub struct HealthApi;

/// Check API liveness.
#[utoipa::path(
    get,
    path = "/",
    tag = "health",
    responses(
        (status = 200, description = "API is healthy", body = String)
    )
)]
pub async fn health_check() -> &'static str {
    "ok"
}
