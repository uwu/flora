use crate::{
    discord_handler::DiscordHandler,
    runtime::BotRuntime,
    services::{
        custom_bots::{CustomBotService, CustomBotWithToken, custom_bot_deployment_id},
        deployments::{Deployment, DeploymentService},
        discord_rest::{DiscordRest, RestConfig},
        orchestrator::FeatureAuthorizationRequest,
        scope_cache::ScopeCache,
    },
};
use color_eyre::eyre::{Context, Result, eyre};
use fred::prelude::Pool;
use parking_lot::{Mutex, RwLock};
use serenity::{
    all::{Client, GatewayIntents, Token},
    http::Http,
};
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use uuid::Uuid;

struct CustomBotClientHandle {
    shutdown: Option<Box<dyn FnOnce() -> bool + Send>>,
    _join: JoinHandle<()>,
}

#[derive(Clone)]
pub struct CustomBotGatewayManager {
    runtime: Arc<BotRuntime>,
    deployments: DeploymentService,
    bots: CustomBotService,
    cache: Pool,
    rest_config: RestConfig,
    clients: Arc<Mutex<HashMap<Uuid, CustomBotClientHandle>>>,
    rests: Arc<RwLock<HashMap<Uuid, Arc<DiscordRest>>>>,
}

impl CustomBotGatewayManager {
    pub fn new(
        runtime: Arc<BotRuntime>,
        deployments: DeploymentService,
        bots: CustomBotService,
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
            clients: Arc::new(Mutex::new(HashMap::new())),
            rests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn is_running(&self, bot_id: Uuid) -> bool {
        self.clients.lock().contains_key(&bot_id)
    }

    pub async fn activate(&self, bot_id: Uuid, deployment: Deployment) -> Result<()> {
        let record = self
            .bots
            .get_with_token(bot_id)
            .await?
            .ok_or_else(|| eyre!("custom bot not found"))?;
        let (rest, token) = self.connection(&record).await?;
        self.runtime
            .deploy_custom_bot_script(&bot_id.to_string(), deployment, rest.clone())
            .await
            .map_err(|err| eyre!(err.to_string()))?;
        if self.clients.lock().contains_key(&bot_id) {
            return Ok(());
        }

        let http = rest.http().clone();
        let app_info = http.get_current_application_info().await?;
        http.set_application_id(app_info.id);
        let handler = Arc::new(DiscordHandler {
            runtime: self.runtime.clone(),
            rest,
            http,
            application_id: Arc::new(std::sync::RwLock::new(Some(app_info.id))),
            deployments: self.deployments.clone(),
            custom_bot_id: Some(bot_id.to_string()),
            bound_guild_id: None,
            server_bot_guilds: Default::default(),
        });
        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS
            | GatewayIntents::DIRECT_MESSAGE_REACTIONS;
        let mut client = Client::builder(token, intents)
            .event_handler(handler)
            .await
            .context("build custom bot Discord client")?;
        let shutdown = client.shard_manager.get_shutdown_trigger();
        let bot_id_for_task = bot_id;
        let join = tokio::spawn(async move {
            if let Err(err) = client.start().await {
                error!(target: "flora:custom-bots", bot_id = %bot_id_for_task, ?err, "custom bot gateway exited");
            }
        });
        self.clients.lock().insert(
            bot_id,
            CustomBotClientHandle {
                shutdown: Some(Box::new(shutdown)),
                _join: join,
            },
        );
        info!(target: "flora:custom-bots", bot_id = %bot_id, "custom bot activated");
        Ok(())
    }

    pub async fn deactivate(&self, bot_id: Uuid) -> Result<()> {
        let handle = self.clients.lock().remove(&bot_id);
        if let Some(mut handle) = handle
            && let Some(shutdown) = handle.shutdown.take()
        {
            let _ = shutdown();
        }
        self.rests.write().remove(&bot_id);
        self.runtime
            .undeploy_custom_bot_script(&bot_id.to_string())
            .await
            .map_err(|err| eyre!(err.to_string()))?;
        Ok(())
    }

    pub async fn reconcile_all(&self) -> Result<()> {
        for record in self.bots.list_with_tokens().await? {
            let allowed = self
                .runtime
                .authorize_feature(FeatureAuthorizationRequest {
                    feature: "custom_bots".to_string(),
                    user_id: record.bot.owner_user_id.clone(),
                    metadata: None,
                })
                .await
                .unwrap_or(false);
            let scope_id = custom_bot_deployment_id(record.bot.id);
            let deployment = self.deployments.get_persisted_deployment(&scope_id).await?;
            if allowed {
                if let Some(deployment) = deployment
                    && let Err(err) = self.activate(record.bot.id, deployment).await
                {
                    warn!(target: "flora:custom-bots", bot_id = %record.bot.id, ?err, "failed to activate custom bot");
                }
            } else if let Err(err) = self.deactivate(record.bot.id).await {
                warn!(target: "flora:custom-bots", bot_id = %record.bot.id, ?err, "failed to deactivate custom bot");
            }
        }
        Ok(())
    }

    async fn connection(&self, record: &CustomBotWithToken) -> Result<(Arc<DiscordRest>, Token)> {
        let token: Token = record
            .token
            .parse()
            .map_err(|err: serenity::secrets::TokenError| eyre!(err))?;
        if let Some(rest) = self.rests.read().get(&record.bot.id).cloned() {
            return Ok((rest, token));
        }
        let http = Arc::new(Http::new(token.clone()));
        http.set_application_id(
            record
                .bot
                .application_id
                .parse::<u64>()
                .map_err(|_| eyre!("invalid custom bot application id"))?
                .into(),
        );
        let scope_cache = ScopeCache::new(http.clone(), self.cache.clone());
        let rest = Arc::new(DiscordRest::new(http, scope_cache, self.rest_config));
        self.rests.write().insert(record.bot.id, rest.clone());
        Ok((rest, token))
    }
}
