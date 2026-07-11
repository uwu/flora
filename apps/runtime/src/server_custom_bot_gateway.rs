use crate::{
    discord_handler::DiscordHandler,
    runtime::BotRuntime,
    services::{
        deployments::{Deployment, DeploymentService},
        discord_rest::{DiscordRest, RestConfig},
        orchestrator::FeatureAuthorizationRequest,
        scope_cache::ScopeCache,
        server_custom_bots::{ServerCustomBotService, ServerCustomBotWithToken},
    },
};
use color_eyre::eyre::{Context, Result, eyre};
use fred::prelude::Pool;
use parking_lot::{Mutex, RwLock};
use serenity::{
    all::{Client, GatewayIntents, GuildId, Token},
    http::Http,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

#[derive(Clone, Default)]
pub struct ServerBotGuildRegistry(Arc<RwLock<HashSet<String>>>);

impl ServerBotGuildRegistry {
    pub fn contains(&self, guild_id: &str) -> bool {
        self.0.read().contains(guild_id)
    }
    fn insert(&self, guild_id: String) {
        self.0.write().insert(guild_id);
    }
    fn remove(&self, guild_id: &str) {
        self.0.write().remove(guild_id);
    }
}

struct ServerBotClientHandle {
    shutdown: Option<Box<dyn FnOnce() -> bool + Send>>,
    join: JoinHandle<()>,
}

#[derive(Clone)]
pub struct ServerCustomBotGatewayManager {
    runtime: Arc<BotRuntime>,
    deployments: DeploymentService,
    bots: ServerCustomBotService,
    cache: Pool,
    rest_config: RestConfig,
    clients: Arc<Mutex<HashMap<String, ServerBotClientHandle>>>,
    rests: Arc<RwLock<HashMap<String, Arc<DiscordRest>>>>,
    registry: ServerBotGuildRegistry,
}

impl ServerCustomBotGatewayManager {
    pub fn new(
        runtime: Arc<BotRuntime>,
        deployments: DeploymentService,
        bots: ServerCustomBotService,
        cache: Pool,
        rest_timeout_ms: u64,
        guild_concurrency: usize,
    ) -> Self {
        Self {
            runtime,
            deployments,
            bots,
            cache,
            rest_config: RestConfig {
                max_wait: Duration::from_millis(rest_timeout_ms),
                guild_concurrency: guild_concurrency.max(1),
            },
            clients: Default::default(),
            rests: Default::default(),
            registry: Default::default(),
        }
    }

    pub fn registry(&self) -> ServerBotGuildRegistry {
        self.registry.clone()
    }
    pub fn is_running(&self, guild_id: &str) -> bool {
        self.clients
            .lock()
            .get(guild_id)
            .is_some_and(|handle| !handle.join.is_finished())
    }

    pub async fn deploy_guild_script(&self, deployment: Deployment) -> Result<()> {
        let rest = { self.rests.read().get(&deployment.guild_id).cloned() };
        match rest {
            Some(rest) => {
                self.runtime
                    .deploy_guild_script_with_rest(deployment, rest)
                    .await
            }
            None => self.runtime.deploy_guild_script(deployment).await,
        }
        .map_err(|err| eyre!(err.to_string()))
    }

    pub async fn activate(&self, guild_id: &str) -> Result<()> {
        let record = self
            .bots
            .get_with_token(guild_id)
            .await?
            .ok_or_else(|| eyre!("server custom bot not found"))?;
        let deployment = self
            .deployments
            .get_persisted_deployment(guild_id)
            .await?
            .ok_or_else(|| eyre!("guild deployment not found"))?;
        let (rest, token) = self.connection(&record)?;
        let guild = GuildId::new(guild_id.parse().context("invalid guild id")?);
        rest.http()
            .get_guild(guild)
            .await
            .context("custom bot is not in this guild")?;

        let handler = Arc::new(DiscordHandler {
            runtime: self.runtime.clone(),
            rest: rest.clone(),
            http: rest.http().clone(),
            application_id: Arc::new(std::sync::RwLock::new(Some(
                record.bot.application_id.parse::<u64>()?.into(),
            ))),
            deployments: self.deployments.clone(),
            custom_bot_id: None,
            bound_guild_id: Some(guild_id.to_string()),
            server_bot_guilds: self.registry.clone(),
        });
        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS;
        let mut client = Client::builder(token, intents)
            .event_handler(handler)
            .await
            .context("build server custom bot client")?;
        let shutdown = client.shard_manager.get_shutdown_trigger();

        let old = { self.clients.lock().remove(guild_id) };
        if let Some(old) = old {
            stop_client(old).await;
        }

        let no_commands: Vec<serenity::builder::CreateCommand<'static>> = Vec::new();
        self.runtime
            .discord_rest()
            .http()
            .create_guild_commands(guild, &no_commands)
            .await
            .context("clear central Flora guild commands")?;
        self.runtime
            .deploy_guild_script_with_rest(deployment, rest.clone())
            .await
            .map_err(|err| eyre!(err.to_string()))?;
        self.registry.insert(guild_id.to_string());

        let guild_id_for_task = guild_id.to_string();
        let join = tokio::spawn(async move {
            if let Err(err) = client.start().await {
                error!(target: "flora:server-custom-bots", guild_id = guild_id_for_task, ?err, "server custom bot gateway exited");
            }
        });
        self.clients.lock().insert(
            guild_id.to_string(),
            ServerBotClientHandle {
                shutdown: Some(Box::new(shutdown)),
                join,
            },
        );
        self.rests.write().insert(guild_id.to_string(), rest);
        info!(target: "flora:server-custom-bots", guild_id, "server custom bot activated");
        Ok(())
    }

    pub async fn deactivate(&self, guild_id: &str) -> Result<()> {
        let rest = { self.rests.write().remove(guild_id) };
        let rest = match rest {
            Some(rest) => Some(rest),
            None => self
                .bots
                .get_with_token(guild_id)
                .await?
                .and_then(|record| self.connection(&record).ok().map(|value| value.0)),
        };
        if let Some(rest) = rest {
            let guild = GuildId::new(guild_id.parse().context("invalid guild id")?);
            let no_commands: Vec<serenity::builder::CreateCommand<'static>> = Vec::new();
            if let Err(err) = rest.http().create_guild_commands(guild, &no_commands).await {
                warn!(target: "flora:server-custom-bots", guild_id, ?err, "failed to clear custom bot commands");
            }
        }
        let handle = { self.clients.lock().remove(guild_id) };
        if let Some(handle) = handle {
            stop_client(handle).await;
        }
        if let Some(deployment) = self.deployments.get_persisted_deployment(guild_id).await? {
            self.runtime
                .deploy_guild_script(deployment)
                .await
                .map_err(|err| eyre!(err.to_string()))?;
        }
        self.registry.remove(guild_id);
        Ok(())
    }

    pub async fn reconcile_all(&self) -> Result<()> {
        for record in self.bots.list_with_tokens().await? {
            let allowed = self
                .runtime
                .authorize_feature(FeatureAuthorizationRequest {
                    feature: "server_custom_bots".to_string(),
                    user_id: record.bot.configured_by_user_id.clone(),
                    metadata: Some(serde_json::json!({ "guildId": record.bot.guild_id })),
                })
                .await
                .unwrap_or(false);
            if allowed {
                if let Err(err) = self.activate(&record.bot.guild_id).await {
                    warn!(target: "flora:server-custom-bots", guild_id = record.bot.guild_id, ?err, "failed to activate server custom bot");
                }
            } else if let Err(err) = self.deactivate(&record.bot.guild_id).await {
                warn!(target: "flora:server-custom-bots", guild_id = record.bot.guild_id, ?err, "failed to deactivate server custom bot");
            }
        }
        Ok(())
    }

    fn connection(&self, record: &ServerCustomBotWithToken) -> Result<(Arc<DiscordRest>, Token)> {
        let token: Token = record
            .token
            .parse()
            .map_err(|err: serenity::secrets::TokenError| eyre!(err))?;
        let http = Arc::new(Http::new(token.clone()));
        http.set_application_id(record.bot.application_id.parse::<u64>()?.into());
        let rest = Arc::new(DiscordRest::new(
            http.clone(),
            ScopeCache::new(http, self.cache.clone()),
            self.rest_config,
        ));
        Ok((rest, token))
    }
}

async fn stop_client(mut handle: ServerBotClientHandle) {
    if let Some(shutdown) = handle.shutdown.take() {
        let _ = shutdown();
    }
    if tokio::time::timeout(Duration::from_secs(5), &mut handle.join)
        .await
        .is_err()
    {
        handle.join.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::ServerBotGuildRegistry;

    #[test]
    fn registry_tracks_guild_identity_overrides() {
        let registry = ServerBotGuildRegistry::default();
        assert!(!registry.contains("123"));
        registry.insert("123".to_string());
        assert!(registry.contains("123"));
        registry.remove("123");
        assert!(!registry.contains("123"));
    }
}
