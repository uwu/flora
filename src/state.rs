use std::sync::Arc;

use crate::{
    auth::AuthService, deployments::DeploymentService, runtime::BotRuntime, tokens::TokenService,
};
use serenity::http::Http;

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
    /// Bot HTTP client for guild permission checks.
    pub http: Arc<Http>,
}
