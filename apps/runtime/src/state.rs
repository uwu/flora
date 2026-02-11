use crate::{
    auth::AuthService, bot_gateway::BotGatewayManager, bot_router::GuildBotRouter,
    deployments::DeploymentService, guild_bots::GuildBotService, kv::KvService,
    runtime::BotRuntime, secrets::SecretService, tokens::TokenService,
};
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
    /// Per-guild BYOB bindings and credential metadata.
    pub guild_bots: GuildBotService,
    /// Runtime router for deciding which bot handles each guild.
    pub bot_router: Arc<GuildBotRouter>,
    /// BYOB gateway manager for dynamic client lifecycle.
    pub bot_gateway: Arc<BotGatewayManager>,
}
