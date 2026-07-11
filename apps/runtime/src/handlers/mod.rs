use axum::{Json, Router, extract::DefaultBodyLimit, routing::get};
use tower_http::compression::CompressionLayer;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::state::AppState;

pub mod auth;
pub mod builds;
pub mod custom_bots;
pub mod deployments;
#[cfg(debug_assertions)]
pub mod dev;
pub mod error;
pub mod guilds;
pub mod health;
pub mod kv;
pub mod logs;
pub mod metrics;
pub mod orchestrator;
pub mod response;
pub mod secrets;
pub mod server_custom_bots;
pub mod tokens;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "flora API",
        description = "Manages @Flora guild deployments, User Bots, Server Custom Bots, authentication, builds, secrets, key-value stores, logs, and operator orchestration."
    ),
    nest(
        (path = "/auth", api = auth::AuthApi),
        (path = "/builds", api = builds::BuildApi),
        (path = "/custom-bots", api = custom_bots::CustomBotsApi),
        (path = "/server-custom-bots", api = server_custom_bots::ServerCustomBotsApi),
        (path = "/guilds", api = guilds::GuildApi),
        (path = "/tokens", api = tokens::TokenApi),
        (path = "/deployments", api = deployments::DeploymentApi),
        (path = "/kv", api = kv::KvApi),
        (path = "/secrets", api = secrets::SecretsApi),
        (path = "/health", api = health::HealthApi),
        (path = "/metrics", api = metrics::MetricsApi),
        (path = "/orchestrator", api = orchestrator::OrchestratorApi),
        (path = "/logs", api = logs::LogsApi)
    ),
    components(schemas(error::ProblemDetails, error::ProblemViolation)),
    tags((name = "flora", description = "flora bot runtime API")),
    servers((url = "/api", description = "flora API base path"))
)]
pub struct ApiDoc;

/// Returns the generated OpenAPI document for the runtime API.
pub fn openapi_document() -> utoipa::openapi::OpenApi {
    let mut document = ApiDoc::openapi();
    apply_problem_responses(&mut document);
    document
}

fn apply_problem_responses(document: &mut utoipa::openapi::OpenApi) {
    use utoipa::openapi::{Content, Ref, RefOr, ResponseBuilder, path::Operation};

    let content = || Content::new(Some(Ref::from_schema_name("ProblemDetails")));
    for path_item in document.paths.paths.values_mut() {
        let operations = [
            &mut path_item.get,
            &mut path_item.put,
            &mut path_item.post,
            &mut path_item.delete,
            &mut path_item.options,
            &mut path_item.head,
            &mut path_item.patch,
            &mut path_item.trace,
        ];
        for operation in operations.into_iter().flatten() {
            normalize_problem_responses(operation, &content);
        }
    }

    fn normalize_problem_responses(operation: &mut Operation, content: &impl Fn() -> Content) {
        for (status, response) in &mut operation.responses.responses {
            if !(status.starts_with('4') || status.starts_with('5')) {
                continue;
            }
            let RefOr::T(response) = response else {
                continue;
            };
            response.content.clear();
            response
                .content
                .insert("application/problem+json".to_string(), content());
        }
        operation.responses.responses.entry("default".to_string()).or_insert_with(|| {
            RefOr::T(
                ResponseBuilder::new()
                    .description("The request failed. Inspect the HTTP status and Problem Details response for recovery guidance.")
                    .content("application/problem+json", content())
                    .build(),
            )
        });
    }
}

/// Build the top-level router with API routes and interactive docs.
pub fn create_router() -> Router<AppState> {
    let compressed_api = Router::new()
        .nest("/auth", auth::router())
        .nest("/builds", builds::router())
        .nest("/custom-bots", custom_bots::router())
        .nest("/server-custom-bots", server_custom_bots::router())
        .nest("/guilds", guilds::router())
        .nest("/tokens", tokens::router())
        .nest("/deployments", deployments::router())
        .nest("/secrets", secrets::router())
        .nest("/kv", kv::router())
        .nest("/orchestrator", orchestrator::router())
        .route("/health", get(health::health_check))
        .route("/metrics", get(metrics::get_metrics))
        .route("/metrics/json", get(metrics::get_metrics_json))
        .layer(CompressionLayer::new());

    let logs_router = Router::new()
        .route("/logs/{guild_id}", get(logs::get_guild_logs))
        .route("/logs/{guild_id}/stream", get(logs::stream_guild_logs));

    let api_router = Router::new().merge(compressed_api).merge(logs_router);

    let oapi_router = Router::new()
        .merge(Scalar::with_url("/scalar", openapi_document()))
        .route("/openapi.json", get(|| async { Json(openapi_document()) }));

    let router = Router::new()
        .nest("/api-docs", oapi_router)
        // Expose API only under `/api/*`
        .nest("/api", api_router)
        .layer(DefaultBodyLimit::max(8 * 1024 * 1024));

    #[cfg(not(debug_assertions))]
    {
        use static_serve::{embed_asset, embed_assets};

        embed_assets!(
            "apps/frontend/dist",
            compress = true,
            cache_busted_paths = ["assets"],
            allow_unknown_extensions = true
        );
        let index_handler = embed_asset!("apps/frontend/dist/index.html", compress = true);

        return router.merge(static_router()).fallback(index_handler);
    }

    #[cfg(debug_assertions)]
    {
        router.nest("/__dev", dev::router())
    }
}

#[cfg(test)]
mod tests {
    use utoipa::openapi::{RefOr, path::Operation};

    use super::openapi_document;

    #[test]
    fn openapi_operations_are_fully_documented() {
        let document = openapi_document();
        for (path, path_item) in &document.paths.paths {
            let operations = [
                path_item.get.as_ref(),
                path_item.put.as_ref(),
                path_item.post.as_ref(),
                path_item.delete.as_ref(),
                path_item.options.as_ref(),
                path_item.head.as_ref(),
                path_item.patch.as_ref(),
                path_item.trace.as_ref(),
            ];
            for operation in operations.into_iter().flatten() {
                assert_documented(path, operation);
            }
        }
    }

    #[test]
    fn generated_openapi_document_is_current() {
        let generated = serde_json::to_value(openapi_document()).unwrap();
        let committed = serde_json::from_str::<serde_json::Value>(include_str!(
            "../../../../openapi/flora.openapi.json"
        ))
        .unwrap();
        assert_eq!(generated, committed);
    }

    fn assert_documented(path: &str, operation: &Operation) {
        assert!(
            operation
                .operation_id
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
            "{path} is missing operationId"
        );
        assert!(
            operation
                .summary
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
            "{path} is missing summary"
        );
        assert!(
            operation
                .description
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
            "{path} is missing description"
        );
        let problem = operation
            .responses
            .responses
            .get("default")
            .expect("default Problem response");
        let RefOr::T(problem) = problem else {
            panic!("{path} default response must be inline")
        };
        assert!(
            problem.content.contains_key("application/problem+json"),
            "{path} default response is not Problem Details"
        );
    }
}
