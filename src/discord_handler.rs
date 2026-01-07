use std::sync::Arc;

use color_eyre::{Report, eyre::eyre};
use flora_macros::expose_payload;
use serenity::all::{
    ApplicationId, ChannelId, CommandInteraction, Context, EventHandler, Guild, GuildId,
    Interaction, Message, MessageId, MessageUpdateEvent, Ready, async_trait,
};
use tracing::{error, info};

use crate::{
    bundler::{DeploymentFile, bundle_files},
    deployments::DeploymentService,
    runtime::BotRuntime,
};

#[derive(Clone)]
pub struct DiscordHandler {
    pub runtime: Arc<BotRuntime>,
    pub http: Arc<serenity::http::Http>,
    pub application_id: Arc<std::sync::RwLock<Option<ApplicationId>>>,
    pub deployments: DeploymentService,
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);

        {
            let mut app_id = self.application_id.write().unwrap();
            *app_id = Some(ready.application.id);
        }
        self.http.set_application_id(ready.application.id);

        for guild in &ready.guilds {
            if let Err(err) = self.register_guild_commands(guild.id).await {
                error!("failed to register guild commands {}: {:?}", guild.id, err);
            }

            if let Err(err) = self.bootstrap_default_script(guild.id).await {
                error!("failed to bootstrap default script for guild {}: {:?}", guild.id, err);
            }
        }

        let payload = ReadyPayload::from(&ready);
        if let Err(err) = self
            .runtime
            .dispatch_js_event("ready", None, serde_json::to_value(payload).unwrap_or_default())
            .await
        {
            error!("dispatch_js_event (ready) error: {:?}", err);
        }
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        info!(
            target: "flora:discord",
            "message event channel={} author={} content={}",
            msg.channel_id,
            msg.author.id,
            msg.content
        );
        let payload = MessagePayload::from(&msg);
        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message payload: {:?}", err);
                return;
            }
        };

        let guild_id = msg.guild_id.map(|guild| guild.get().to_string());
        if let Err(err) = self.runtime.dispatch_js_event("messageCreate", guild_id, value).await {
            error!("dispatch_js_event error: {:?}", err);
        }
    }

    async fn message_update(
        &self,
        _ctx: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let payload = MessageUpdatePayload::from_parts(old, new, &event);
        let guild_id = payload.guild_id.clone();
        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message update payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageUpdate", guild_id, value).await {
            error!("dispatch_js_event (messageUpdate) error: {:?}", err);
        }
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        let payload = MessageDeletePayload {
            id: deleted_message_id.get().to_string(),
            channel_id: channel_id.get().to_string(),
            guild_id: guild_id.map(|g| g.get().to_string()),
        };
        let guild_id = payload.guild_id.clone();

        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message delete payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageDelete", guild_id, value).await {
            error!("dispatch_js_event (messageDelete) error: {:?}", err);
        }
    }

    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        multiple_deleted_messages_ids: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) {
        let payload = MessageDeleteBulkPayload {
            ids: multiple_deleted_messages_ids.into_iter().map(|id| id.get().to_string()).collect(),
            channel_id: channel_id.get().to_string(),
            guild_id: guild_id.map(|g| g.get().to_string()),
        };
        let guild_id = payload.guild_id.clone();

        let value = match serde_json::to_value(payload) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to serialize message bulk delete payload: {:?}", err);
                return;
            }
        };

        if let Err(err) = self.runtime.dispatch_js_event("messageDeleteBulk", guild_id, value).await
        {
            error!("dispatch_js_event (messageDeleteBulk) error: {:?}", err);
        }
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                info!(
                    target: "flora:discord",
                    "slash command interaction guild={:?} channel={} name={}",
                    command.guild_id,
                    command.channel_id,
                    command.data.name
                );

                let payload = InteractionCreatePayload::from(&command);
                let guild_id = payload.guild_id.clone();
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize interaction payload: {:?}", err);
                        return;
                    }
                };

                if let Err(err) =
                    self.runtime.dispatch_js_event("interactionCreate", guild_id, value).await
                {
                    error!("dispatch_js_event (interactionCreate) error: {:?}", err);
                }
            }
            _ => {}
        }
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        // Bootstrap a starter script when the bot joins a guild and no deployment exists yet.
        if let Err(err) = self.bootstrap_default_script(guild.id).await {
            error!(target: "flora:deployments", guild_id = guild.id.get(), ?err, "failed to bootstrap default script on guild create");
        }
    }
}

#[expose_payload(from = "serenity::all::User")]
struct UserPayload {
    #[expose(expr = "src.id.get().to_string()")]
    id: String,
    #[expose(expr = "src.name.clone()")]
    username: String,
    #[expose(expr = "src.discriminator.map(|d| d.get())")]
    discriminator: Option<u16>,
    #[expose(expr = "src.bot")]
    bot: bool,
}

#[expose_payload]
struct MemberPayload {
    user: UserPayload,
    nick: Option<String>,
    avatar: Option<String>,
    roles: Vec<String>,
    joined_at: Option<String>,
    premium_since: Option<String>,
    deaf: bool,
    mute: bool,
    flags: u32,
    pending: bool,
    permissions: Option<String>,
    communication_disabled_until: Option<String>,
}

#[expose_payload]
struct MessagePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
    content: String,
    author: UserPayload,
    member: Option<MemberPayload>,
}

impl From<&Message> for MessagePayload {
    fn from(msg: &Message) -> Self {
        let member = msg.member.as_ref().map(|member| MemberPayload {
            user: UserPayload {
                id: msg.author.id.get().to_string(),
                username: msg.author.name.clone(),
                discriminator: msg.author.discriminator.map(|d| d.get()),
                bot: msg.author.bot,
            },
            nick: member.nick.clone(),
            avatar: None,
            roles: member.roles.iter().map(|role| role.get().to_string()).collect(),
            joined_at: member.joined_at.and_then(|ts| ts.to_rfc3339()),
            premium_since: None,
            deaf: member.deaf,
            mute: member.mute,
            flags: 0,
            pending: member.pending,
            permissions: None,
            communication_disabled_until: None,
        });
        Self {
            id: msg.id.get().to_string(),
            channel_id: msg.channel_id.get().to_string(),
            guild_id: msg.guild_id.map(|g| g.get().to_string()),
            content: msg.content.clone(),
            author: UserPayload {
                id: msg.author.id.get().to_string(),
                username: msg.author.name.clone(),
                discriminator: msg.author.discriminator.map(|d| d.get()),
                bot: msg.author.bot,
            },
            member,
        }
    }
}

#[expose_payload]
struct MessageUpdatePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
    content: Option<String>,
    author: Option<UserPayload>,
    edited_timestamp: Option<String>,
    old: Option<MessagePayload>,
    new: Option<MessagePayload>,
}

impl MessageUpdatePayload {
    fn from_parts(old: Option<Message>, new: Option<Message>, event: &MessageUpdateEvent) -> Self {
        let guild_id = event
            .guild_id
            .map(|g| g.get().to_string())
            .or_else(|| new.as_ref().and_then(|m| m.guild_id).map(|g| g.get().to_string()))
            .or_else(|| old.as_ref().and_then(|m| m.guild_id).map(|g| g.get().to_string()));

        let content = event.content.clone().or_else(|| new.as_ref().map(|m| m.content.clone()));

        let author = event
            .author
            .as_ref()
            .map(UserPayload::from)
            .or_else(|| new.as_ref().map(|m| UserPayload::from(&m.author)));

        let edited_timestamp =
            event.edited_timestamp.and_then(|ts| ts.to_rfc3339()).or_else(|| {
                new.as_ref().and_then(|m| m.edited_timestamp.and_then(|ts| ts.to_rfc3339()))
            });

        Self {
            id: event.id.get().to_string(),
            channel_id: event.channel_id.get().to_string(),
            guild_id,
            content,
            author,
            edited_timestamp,
            old: old.as_ref().map(MessagePayload::from),
            new: new.as_ref().map(MessagePayload::from),
        }
    }
}

#[expose_payload]
struct MessageDeletePayload {
    id: String,
    channel_id: String,
    guild_id: Option<String>,
}

#[expose_payload]
struct MessageDeleteBulkPayload {
    ids: Vec<String>,
    channel_id: String,
    guild_id: Option<String>,
}

#[expose_payload]
struct ReadyPayload {
    user: UserPayload,
    guild_ids: Vec<String>,
}

#[expose_payload]
struct InteractionCreatePayload {
    interaction_id: String,
    interaction_token: String,
    application_id: String,
    guild_id: Option<String>,
    channel_id: Option<String>,
    user: UserPayload,
    member: Option<MemberPayload>,
    command_name: String,
    data: serde_json::Value,
    locale: Option<String>,
    guild_locale: Option<String>,
}

impl From<&CommandInteraction> for InteractionCreatePayload {
    fn from(interaction: &CommandInteraction) -> Self {
        let data = serde_json::to_value(&interaction.data).unwrap_or_default();
        Self {
            interaction_id: interaction.id.get().to_string(),
            interaction_token: interaction.token.clone(),
            application_id: interaction.application_id.get().to_string(),
            guild_id: interaction.guild_id.map(|g| g.get().to_string()),
            channel_id: Some(interaction.channel_id.get().to_string()),
            user: UserPayload::from(&interaction.user),
            member: interaction.member.as_ref().map(|m| MemberPayload {
                user: UserPayload::from(&interaction.user),
                nick: m.nick.clone(),
                avatar: m.avatar.map(|h| h.to_string()),
                roles: m.roles.iter().map(|r| r.get().to_string()).collect(),
                joined_at: m.joined_at.and_then(|ts| ts.to_rfc3339()),
                premium_since: m.premium_since.and_then(|ts| ts.to_rfc3339()),
                deaf: m.deaf,
                mute: m.mute,
                flags: m.flags.bits(),
                pending: m.pending,
                permissions: m.permissions.map(|p| format!("{:?}", p)),
                communication_disabled_until: m
                    .communication_disabled_until
                    .and_then(|ts| ts.to_rfc3339()),
            }),
            command_name: interaction.data.name.clone(),
            data,
            locale: Some(interaction.locale.clone()),
            guild_locale: interaction.guild_locale.clone(),
        }
    }
}

impl DiscordHandler {
    async fn register_guild_commands(&self, _guild_id: GuildId) -> serenity::Result<()> {
        // TODO: register user-provided slash commands once surfaced from the JS runtime or
        // deployment metadata. For now, do nothing to avoid polluting guild command space.
        Ok(())
    }

    async fn bootstrap_default_script(&self, guild_id: GuildId) -> Result<(), Report> {
        let guild_str = guild_id.get().to_string();
        if self.deployments.get_deployment(&guild_str).await?.is_some() {
            return Ok(());
        }

        let files = vec![
            DeploymentFile {
                path: DEFAULT_GUILD_ENTRY.to_string(),
                contents: DEFAULT_GUILD_SCRIPT.to_string(),
            },
            DeploymentFile {
                path: "src/utils/reply.ts".to_string(),
                contents: DEFAULT_GUILD_UTILS_REPLY.to_string(),
            },
        ];
        let bundle_name = format!("guild:{guild_str}.bundle.js");
        let bundle = bundle_files(&bundle_name, DEFAULT_GUILD_ENTRY, &files)
            .map_err(|err| eyre!(err.to_string()))?;
        let deployment = self
            .deployments
            .upsert_deployment(
                guild_str.clone(),
                DEFAULT_GUILD_ENTRY.to_string(),
                files,
                bundle.code,
            )
            .await?;
        self.runtime.deploy_guild_script(deployment).await.map_err(|err| eyre!(err.to_string()))?;
        info!(target: "flora:deployments", guild_id = guild_str, "bootstrapped default script");
        Ok(())
    }
}

impl From<&Ready> for ReadyPayload {
    fn from(ready: &Ready) -> Self {
        Self {
            user: UserPayload {
                id: ready.user.id.get().to_string(),
                username: ready.user.name.clone(),
                discriminator: ready.user.discriminator.map(|d| d.get()),
                bot: ready.user.bot,
            },
            guild_ids: ready.guilds.iter().map(|g| g.id.get().to_string()).collect(),
        }
    }
}

// Ship a minimal starter bot to new guilds without deployments.
const DEFAULT_GUILD_ENTRY: &str = "src/main.ts";
const DEFAULT_GUILD_SCRIPT: &str = include_str!("../example/src/main.ts");
const DEFAULT_GUILD_UTILS_REPLY: &str = include_str!("../example/src/utils/reply.ts");
