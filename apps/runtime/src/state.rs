use crate::{
    custom_bot_gateway::CustomBotGatewayManager,
    runtime::BotRuntime,
    server_custom_bot_gateway::ServerCustomBotGatewayManager,
    services::{
        auth::AuthService, build::BuildServiceClient, custom_bots::CustomBotService,
        deployments::DeploymentService, kv::KvService, secrets::SecretService,
        server_custom_bots::ServerCustomBotService, tokens::TokenService,
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
    /// User-owned custom bot metadata and encrypted tokens.
    pub custom_bots: CustomBotService,
    /// Lifecycle manager for custom bot isolates and Discord gateway clients.
    pub custom_bot_gateway: CustomBotGatewayManager,
    /// Guild-owned Discord identity metadata and encrypted tokens.
    pub server_custom_bots: ServerCustomBotService,
    /// Lifecycle manager for guild-owned Discord gateway clients.
    pub server_custom_bot_gateway: ServerCustomBotGatewayManager,
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
    /// Discord user ID allowed to manage the singleton orchestrator deployment.
    pub orchestrator_operator_user_id: Option<String>,
}
