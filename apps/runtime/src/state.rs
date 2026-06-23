use crate::{
    runtime::BotRuntime,
    services::{
        auth::AuthService, build::BuildServiceClient, deployments::DeploymentService,
        kv::KvService, secrets::SecretService, tokens::TokenService,
    },
};
use serenity::http::Http;
use std::sync::Arc;

/// Shared application state injected into all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    /// JavaScript runtime that executes bot code per guild.
    pub runtime: Arc<BotRuntime>,
    /// Service responsible for storing and caching deployment records.
    pub deployments: DeploymentService,
    /// Authentication and session management.
    pub auth: AuthService,
    /// Long-lived API tokens for CLI authentication.
    pub tokens: TokenService,
    /// Key-value store service backed by RocksDB.
    pub kv: KvService,
    /// Secret storage and encryption.
    pub secrets: SecretService,
    /// Build service client for server-side bundling.
    pub build_service: BuildServiceClient,
    /// Bot HTTP client for guild permission checks.
    pub http: Arc<Http>,
    /// Bearer token for operator-only endpoints.
    pub operator_secret: Option<String>,
}
