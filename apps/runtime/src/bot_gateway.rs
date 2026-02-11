use crate::{
    bot_router::GuildBotRouter,
    deployments::DeploymentService,
    discord_handler::DiscordHandler,
    guild_bots::{GuildBotBindingWithToken, GuildBotService},
    runtime::BotRuntime,
};
use color_eyre::eyre::{Context, Result};
use parking_lot::RwLock;
use serenity::all::{Client, GatewayIntents, Token};
use std::{collections::HashMap, sync::Arc};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

struct BotClientHandle {
    _join: JoinHandle<()>,
}

/// Manages additional BYOB gateway clients.
#[derive(Clone)]
pub struct BotGatewayManager {
    runtime: Arc<BotRuntime>,
    deployments: DeploymentService,
    guild_bots: GuildBotService,
    bot_router: Arc<GuildBotRouter>,
    intents: GatewayIntents,
    clients: Arc<RwLock<HashMap<String, BotClientHandle>>>,
}

impl BotGatewayManager {
    pub fn new(
        runtime: Arc<BotRuntime>,
        deployments: DeploymentService,
        guild_bots: GuildBotService,
        bot_router: Arc<GuildBotRouter>,
        intents: GatewayIntents,
    ) -> Self {
        Self {
            runtime,
            deployments,
            guild_bots,
            bot_router,
            intents,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn sync_all_bindings(&self) -> Result<()> {
        let bindings = self.guild_bots.list_bindings_with_tokens().await?;
        self.bot_router.clear_all_guild_bindings();

        for binding in bindings {
            match self.ensure_gateway_for_binding(&binding).await {
                Ok(bot_user_id) => {
                    self.bot_router
                        .set_guild_binding(binding.binding.guild_id, bot_user_id);
                }
                Err(err) => {
                    warn!(
                        target: "flora:discord",
                        guild_id = binding.binding.guild_id,
                        ?err,
                        "failed to start byob gateway for guild"
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn sync_guild_binding(&self, guild_id: &str) -> Result<()> {
        let Some(binding) = self.guild_bots.get_binding_with_token(guild_id).await? else {
            self.bot_router.clear_guild_binding(guild_id);
            info!(target: "flora:discord", guild_id, "removed byob binding from router");
            return Ok(());
        };

        let bot_user_id = self.ensure_gateway_for_binding(&binding).await?;
        self.bot_router
            .set_guild_binding(guild_id.to_string(), bot_user_id.clone());

        info!(target: "flora:discord", guild_id, bot_user_id, "synced byob guild binding");
        Ok(())
    }

    async fn ensure_gateway_for_binding(
        &self,
        binding: &GuildBotBindingWithToken,
    ) -> Result<String> {
        let known_bot_user_id = binding.binding.bot_user_id.clone();
        if self.clients.read().contains_key(&known_bot_user_id) {
            return Ok(known_bot_user_id);
        }

        let token: Token = binding
            .bot_token
            .parse()
            .map_err(|err: serenity::secrets::TokenError| color_eyre::eyre::eyre!(err))
            .context("invalid bot token")?;

        let http = Arc::new(serenity::http::Http::new(token.clone()));
        let bot_user = http
            .get_current_user()
            .await
            .context("failed to load bot user")?;
        let app_info = http
            .get_current_application_info()
            .await
            .context("failed to load bot application")?;
        http.set_application_id(app_info.id);

        let bot_user_id = bot_user.id.get().to_string();
        self.bot_router
            .register_bot_http(bot_user_id.clone(), http.clone());

        if self.clients.read().contains_key(&bot_user_id) {
            return Ok(bot_user_id);
        }

        let handler = Arc::new(DiscordHandler {
            runtime: self.runtime.clone(),
            http,
            application_id: Arc::new(std::sync::RwLock::new(Some(app_info.id))),
            deployments: self.deployments.clone(),
            bot_router: self.bot_router.clone(),
            source_bot_user_id: bot_user_id.clone(),
        });

        let intents = self.intents;
        let mut client = Client::builder(token, intents)
            .event_handler(handler)
            .await
            .context("failed to build byob discord client")?;

        let bot_user_id_for_task = bot_user_id.clone();
        let join = tokio::spawn(async move {
            if let Err(err) = client.start().await {
                error!(
                    target: "flora:discord",
                    bot_user_id = bot_user_id_for_task,
                    ?err,
                    "byob discord client exited"
                );
            }
        });

        self.clients
            .write()
            .entry(bot_user_id.clone())
            .or_insert(BotClientHandle { _join: join });

        info!(target: "flora:discord", bot_user_id, "started byob gateway client");
        Ok(bot_user_id)
    }
}
