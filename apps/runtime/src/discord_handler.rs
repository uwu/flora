use std::sync::Arc;

use color_eyre::{Report, eyre::eyre};
use flora_macros::expose_payload;
use serenity::all::{
    ApplicationId, CommandInteraction, ComponentInteraction, Context, EventHandler, FullEvent,
    GuildId, Interaction, Message, MessageUpdateEvent, ModalInteraction, Reaction, Ready,
    async_trait,
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
    async fn dispatch(&self, _ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready {
                data_about_bot: ready,
                ..
            } => {
                info!("Connected as {}", ready.user.name.to_string());

                {
                    let mut app_id = self.application_id.write().unwrap();
                    *app_id = Some(ready.application.id);
                }
                self.http.set_application_id(ready.application.id);

                for guild in &ready.guilds {
                    if let Err(err) = self.bootstrap_default_script(guild.id).await {
                        error!(
                            "failed to bootstrap default script for guild {}: {:?}",
                            guild.id, err
                        );
                    }
                }

                let payload = EventReady::from(ready);
                if let Err(err) = self
                    .runtime
                    .dispatch_js_event(
                        "ready",
                        None,
                        serde_json::to_value(payload).unwrap_or_default(),
                    )
                    .await
                {
                    error!("dispatch_js_event (ready) error: {:?}", err);
                }
            }
            FullEvent::Message {
                new_message: msg,
                ..
            } => {
                info!(
                    target: "flora:discord",
                    "message event channel={} author={} content={}",
                    msg.channel_id,
                    msg.author.id,
                    msg.content
                );
                let payload = EventMessage::from(msg);
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize message payload: {:?}", err);
                        return;
                    }
                };

                let guild_id = msg.guild_id.map(|guild| guild.get().to_string());
                if guild_id.is_none() {
                    return;
                }
                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("messageCreate", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event error: {:?}", err);
                }
            }
            FullEvent::MessageUpdate {
                old_if_available,
                event,
                ..
            } => {
                let payload = EventMessageUpdate::from_parts(
                    old_if_available.clone(),
                    Some(event.message.clone()),
                    event,
                );
                let guild_id = payload.guild_id.clone();
                if guild_id.is_none() {
                    return;
                }
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize message update payload: {:?}", err);
                        return;
                    }
                };

                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("messageUpdate", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event (messageUpdate) error: {:?}", err);
                }
            }
            FullEvent::MessageDelete {
                channel_id,
                deleted_message_id,
                guild_id,
                ..
            } => {
                let payload = EventMessageDelete {
                    id: deleted_message_id.get().to_string(),
                    channel_id: channel_id.get().to_string(),
                    guild_id: guild_id.map(|g| g.get().to_string()),
                };
                let guild_id = payload.guild_id.clone();
                if guild_id.is_none() {
                    return;
                }

                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize message delete payload: {:?}", err);
                        return;
                    }
                };

                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("messageDelete", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event (messageDelete) error: {:?}", err);
                }
            }
            FullEvent::MessageDeleteBulk {
                channel_id,
                multiple_deleted_messages_ids,
                guild_id,
                ..
            } => {
                let payload = EventMessageDeleteBulk {
                    ids: multiple_deleted_messages_ids
                        .iter()
                        .map(|id| id.get().to_string())
                        .collect(),
                    channel_id: channel_id.get().to_string(),
                    guild_id: guild_id.map(|g| g.get().to_string()),
                };
                let guild_id = payload.guild_id.clone();
                if guild_id.is_none() {
                    return;
                }

                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize message bulk delete payload: {:?}", err);
                        return;
                    }
                };

                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("messageDeleteBulk", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event (messageDeleteBulk) error: {:?}", err);
                }
            }
            FullEvent::InteractionCreate {
                interaction,
                ..
            } => match interaction {
                Interaction::Command(command) => {
                    info!(
                        target: "flora:discord",
                        "slash command interaction guild={:?} channel={} name={}",
                        command.guild_id,
                        command.channel_id,
                        command.data.name
                    );

                    let payload = EventInteractionCreate::from(command);
                    let guild_id = payload.guild_id.clone();
                    if guild_id.is_none() {
                        return;
                    }
                    let value = match serde_json::to_value(payload) {
                        Ok(value) => value,
                        Err(err) => {
                            error!("Failed to serialize interaction payload: {:?}", err);
                            return;
                        }
                    };

                    if let Err(err) = self
                        .runtime
                        .dispatch_js_event("interactionCreate", guild_id, value)
                        .await
                    {
                        error!("dispatch_js_event (interactionCreate) error: {:?}", err);
                    }
                }
                Interaction::Component(component) => {
                    let payload = EventComponentInteraction::from(component);
                    let guild_id = payload.guild_id.clone();
                    if guild_id.is_none() {
                        return;
                    }
                    let value = match serde_json::to_value(payload) {
                        Ok(value) => value,
                        Err(err) => {
                            error!("Failed to serialize component payload: {:?}", err);
                            return;
                        }
                    };
                    if let Err(err) = self
                        .runtime
                        .dispatch_js_event("componentInteraction", guild_id, value)
                        .await
                    {
                        error!("dispatch_js_event (componentInteraction) error: {:?}", err);
                    }
                }
                Interaction::Modal(modal) => {
                    let payload = EventModalSubmit::from(modal);
                    let guild_id = payload.guild_id.clone();
                    if guild_id.is_none() {
                        return;
                    }
                    let value = match serde_json::to_value(payload) {
                        Ok(value) => value,
                        Err(err) => {
                            error!("Failed to serialize modal payload: {:?}", err);
                            return;
                        }
                    };
                    if let Err(err) = self
                        .runtime
                        .dispatch_js_event("modalSubmit", guild_id, value)
                        .await
                    {
                        error!("dispatch_js_event (modalSubmit) error: {:?}", err);
                    }
                }
                _ => {}
            },
            FullEvent::ReactionAdd {
                add_reaction: reaction,
                ..
            } => {
                let payload = EventReaction::from(reaction);
                let guild_id = payload.guild_id.clone();
                if guild_id.is_none() {
                    return;
                }
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize reaction payload: {:?}", err);
                        return;
                    }
                };
                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("reactionAdd", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event (reactionAdd) error: {:?}", err);
                }
            }
            FullEvent::ReactionRemove {
                removed_reaction: reaction,
                ..
            } => {
                let payload = EventReaction::from(reaction);
                let guild_id = payload.guild_id.clone();
                if guild_id.is_none() {
                    return;
                }
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize reaction payload: {:?}", err);
                        return;
                    }
                };
                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("reactionRemove", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event (reactionRemove) error: {:?}", err);
                }
            }
            FullEvent::ReactionRemoveAll {
                guild_id,
                channel_id,
                removed_from_message_id,
                ..
            } => {
                let payload = EventReactionRemoveAll {
                    message_id: removed_from_message_id.get().to_string(),
                    channel_id: channel_id.get().to_string(),
                    guild_id: guild_id.map(|g| g.get().to_string()),
                };
                let guild_id = payload.guild_id.clone();
                if guild_id.is_none() {
                    return;
                }
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize reaction payload: {:?}", err);
                        return;
                    }
                };
                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("reactionRemoveAll", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event (reactionRemoveAll) error: {:?}", err);
                }
            }
            FullEvent::ReactionRemoveEmoji {
                removed_reactions: reaction,
                ..
            } => {
                let payload = EventReaction::from(reaction);
                let guild_id = payload.guild_id.clone();
                if guild_id.is_none() {
                    return;
                }
                let value = match serde_json::to_value(payload) {
                    Ok(value) => value,
                    Err(err) => {
                        error!("Failed to serialize reaction payload: {:?}", err);
                        return;
                    }
                };
                if let Err(err) = self
                    .runtime
                    .dispatch_js_event("reactionRemoveEmoji", guild_id, value)
                    .await
                {
                    error!("dispatch_js_event (reactionRemoveEmoji) error: {:?}", err);
                }
            }
            FullEvent::GuildCreate {
                guild,
                is_new: _,
                ..
            } => {
                // Bootstrap a starter script when the bot joins a guild and no deployment exists yet.
                if let Err(err) = self.bootstrap_default_script(guild.id).await {
                    error!(target: "flora:deployments", guild_id = guild.id.get(), ?err, "failed to bootstrap default script on guild create");
                }
            }
            _ => {}
        }
    }
}

/// A Discord user.
///
/// [Discord docs](https://discord.com/developers/docs/resources/user#user-object).
#[expose_payload(from = "serenity::all::User")]
pub struct EventUser {
    /// The user's unique snowflake ID.
    #[expose(expr = "src.id.get().to_string()")]
    id: String,
    /// The user's username (not unique across Discord).
    #[expose(expr = "src.name.to_string()")]
    username: String,
    /// The user's Discord-tag (discriminator), if any.
    #[expose(expr = "src.discriminator.map(|d| d.get())")]
    discriminator: Option<u16>,
    /// Whether this user is a bot account.
    #[expose(expr = "src.bot()")]
    bot: bool,
}

/// A member of a guild.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-member-object).
#[expose_payload]
pub struct EventMember {
    /// The user this member represents.
    user: EventUser,
    /// The member's guild-specific nickname.
    nick: Option<String>,
    /// The member's guild-specific avatar hash.
    avatar: Option<String>,
    /// Role IDs assigned to this member.
    roles: Vec<String>,
    /// ISO8601 timestamp when the member joined the guild.
    joined_at: Option<String>,
    /// ISO8601 timestamp when the member started boosting the guild.
    premium_since: Option<String>,
    /// Whether the member is deafened in voice channels.
    deaf: bool,
    /// Whether the member is muted in voice channels.
    mute: bool,
    /// Guild member flags.
    flags: u32,
    /// Whether the member has not yet passed the guild's membership screening.
    pending: bool,
    /// Total permissions of the member in the channel (only in interactions).
    permissions: Option<String>,
    /// ISO8601 timestamp when the member's timeout expires.
    communication_disabled_until: Option<String>,
}

/// A message sent in a channel.
///
/// [Discord docs](https://discord.com/developers/docs/resources/message#message-object).
#[expose_payload]
pub struct EventMessage {
    /// The message's unique snowflake ID.
    id: String,
    /// The channel the message was sent in.
    channel_id: String,
    /// The guild the message was sent in (if applicable).
    guild_id: Option<String>,
    /// The message content.
    content: String,
    /// The author of the message.
    author: EventUser,
    /// Member properties of the author (if sent in a guild).
    member: Option<EventMember>,
}

impl From<&Message> for EventMessage {
    fn from(msg: &Message) -> Self {
        let member = msg.member.as_ref().map(|member| EventMember {
            user: EventUser {
                id: msg.author.id.get().to_string(),
                username: msg.author.name.to_string(),
                discriminator: msg.author.discriminator.map(|d| d.get()),
                bot: msg.author.bot(),
            },
            nick: member.nick.as_ref().map(|n| n.to_string()),
            avatar: None,
            roles: member
                .roles
                .iter()
                .map(|role| role.get().to_string())
                .collect(),
            joined_at: member.joined_at.map(|ts| ts.to_rfc3339()),
            premium_since: None,
            deaf: member.deaf(),
            mute: member.mute(),
            flags: 0,
            pending: member.pending(),
            permissions: None,
            communication_disabled_until: None,
        });
        Self {
            id: msg.id.get().to_string(),
            channel_id: msg.channel_id.get().to_string(),
            guild_id: msg.guild_id.map(|g| g.get().to_string()),
            content: msg.content.to_string(),
            author: EventUser {
                id: msg.author.id.get().to_string(),
                username: msg.author.name.to_string(),
                discriminator: msg.author.discriminator.map(|d| d.get()),
                bot: msg.author.bot(),
            },
            member,
        }
    }
}

/// Payload for a message update event.
///
/// [Discord docs](https://discord.com/developers/docs/events/gateway-events#message-update).
#[expose_payload]
pub struct EventMessageUpdate {
    /// The message's unique snowflake ID.
    id: String,
    /// The channel the message was sent in.
    channel_id: String,
    /// The guild the message was sent in (if applicable).
    guild_id: Option<String>,
    /// The new message content.
    content: Option<String>,
    /// The author of the message.
    author: Option<EventUser>,
    /// ISO8601 timestamp when the message was edited.
    edited_timestamp: Option<String>,
    /// The cached old message before the update (if available).
    old: Option<EventMessage>,
    /// The full new message after the update (if available).
    new: Option<EventMessage>,
}

impl EventMessageUpdate {
    fn from_parts(old: Option<Message>, new: Option<Message>, event: &MessageUpdateEvent) -> Self {
        let guild_id = event
            .message
            .guild_id
            .map(|g| g.get().to_string())
            .or_else(|| {
                new.as_ref()
                    .and_then(|m| m.guild_id)
                    .map(|g| g.get().to_string())
            })
            .or_else(|| {
                old.as_ref()
                    .and_then(|m| m.guild_id)
                    .map(|g| g.get().to_string())
            });

        let content = Some(event.message.content.to_string())
            .or_else(|| new.as_ref().map(|m| m.content.to_string()));

        let author = Some(EventUser::from(&event.message.author))
            .or_else(|| new.as_ref().map(|m| EventUser::from(&m.author)));

        let edited_timestamp = event
            .message
            .edited_timestamp
            .map(|ts| ts.to_rfc3339())
            .or_else(|| {
                new.as_ref()
                    .and_then(|m| m.edited_timestamp.map(|ts| ts.to_rfc3339()))
            });

        Self {
            id: event.message.id.get().to_string(),
            channel_id: event.message.channel_id.get().to_string(),
            guild_id,
            content,
            author,
            edited_timestamp,
            old: old.as_ref().map(EventMessage::from),
            new: new.as_ref().map(EventMessage::from),
        }
    }
}

/// Payload for a message delete event.
///
/// [Discord docs](https://discord.com/developers/docs/events/gateway-events#message-delete).
#[expose_payload]
pub struct EventMessageDelete {
    /// The deleted message's ID.
    id: String,
    /// The channel the message was in.
    channel_id: String,
    /// The guild the message was in (if applicable).
    guild_id: Option<String>,
}

/// Payload for a bulk message delete event.
///
/// [Discord docs](https://discord.com/developers/docs/events/gateway-events#message-delete-bulk).
#[expose_payload]
pub struct EventMessageDeleteBulk {
    /// IDs of the deleted messages.
    ids: Vec<String>,
    /// The channel the messages were in.
    channel_id: String,
    /// The guild the messages were in (if applicable).
    guild_id: Option<String>,
}

/// Payload for the ready event, fired when the bot connects to the gateway.
///
/// [Discord docs](https://discord.com/developers/docs/events/gateway-events#ready).
#[expose_payload]
pub struct EventReady {
    /// The bot user.
    user: EventUser,
    /// IDs of guilds the bot is a member of.
    guild_ids: Vec<String>,
}

/// Payload for an application command interaction.
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object).
#[expose_payload]
pub struct EventInteractionCreate {
    /// The interaction's unique snowflake ID.
    interaction_id: String,
    /// Token for responding to this interaction.
    interaction_token: String,
    /// The application's ID.
    application_id: String,
    /// The guild the interaction was triggered in.
    guild_id: Option<String>,
    /// The channel the interaction was triggered in.
    channel_id: Option<String>,
    /// The user who triggered the interaction.
    user: EventUser,
    /// Member properties of the user (if triggered in a guild).
    member: Option<EventMember>,
    /// Name of the invoked command.
    command_name: String,
    /// Raw interaction data payload.
    data: serde_json::Value,
    /// The user's locale.
    locale: Option<String>,
    /// The guild's preferred locale.
    guild_locale: Option<String>,
}

impl From<&CommandInteraction> for EventInteractionCreate {
    fn from(interaction: &CommandInteraction) -> Self {
        let data = serde_json::to_value(&interaction.data).unwrap_or_default();
        Self {
            interaction_id: interaction.id.get().to_string(),
            interaction_token: interaction.token.to_string(),
            application_id: interaction.application_id.get().to_string(),
            guild_id: interaction.guild_id.map(|g| g.get().to_string()),
            channel_id: Some(interaction.channel_id.get().to_string()),
            user: EventUser::from(&interaction.user),
            member: interaction.member.as_ref().map(|m| EventMember {
                user: EventUser::from(&interaction.user),
                nick: m.nick.as_ref().map(|n| n.to_string()),
                avatar: m.avatar.map(|h| h.to_string()),
                roles: m.roles.iter().map(|r| r.get().to_string()).collect(),
                joined_at: m.joined_at.map(|ts| ts.to_rfc3339()),
                premium_since: m.premium_since.map(|ts| ts.to_rfc3339()),
                deaf: m.deaf(),
                mute: m.mute(),
                flags: m.flags.bits(),
                pending: m.pending(),
                permissions: m.permissions.map(|p| format!("{:?}", p)),
                communication_disabled_until: m
                    .communication_disabled_until
                    .map(|ts| ts.to_rfc3339()),
            }),
            command_name: interaction.data.name.to_string(),
            data,
            locale: Some(interaction.locale.to_string()),
            guild_locale: interaction.guild_locale.as_ref().map(|l| l.to_string()),
        }
    }
}

/// Payload for a message component interaction (buttons, select menus).
///
/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object).
#[expose_payload]
pub struct EventComponentInteraction {
    /// The interaction's unique snowflake ID.
    interaction_id: String,
    /// Token for responding to this interaction.
    interaction_token: String,
    /// The application's ID.
    application_id: String,
    /// The guild the interaction was triggered in.
    guild_id: Option<String>,
    /// The channel the interaction was triggered in.
    channel_id: Option<String>,
    /// The user who triggered the interaction.
    user: EventUser,
    /// Member properties of the user (if triggered in a guild).
    member: Option<EventMember>,
    /// Raw component interaction data payload.
    data: serde_json::Value,
    /// The user's locale.
    locale: Option<String>,
    /// The guild's preferred locale.
    guild_locale: Option<String>,
    /// ID of the message the component is attached to.
    message_id: Option<String>,
}

impl From<&ComponentInteraction> for EventComponentInteraction {
    fn from(interaction: &ComponentInteraction) -> Self {
        let data = serde_json::to_value(&interaction.data).unwrap_or_default();
        Self {
            interaction_id: interaction.id.get().to_string(),
            interaction_token: interaction.token.to_string(),
            application_id: interaction.application_id.get().to_string(),
            guild_id: interaction.guild_id.map(|g| g.get().to_string()),
            channel_id: Some(interaction.channel_id.get().to_string()),
            user: EventUser::from(&interaction.user),
            member: interaction.member.as_ref().map(|m| EventMember {
                user: EventUser::from(&interaction.user),
                nick: m.nick.as_ref().map(|n| n.to_string()),
                avatar: m.avatar.map(|h| h.to_string()),
                roles: m.roles.iter().map(|r| r.get().to_string()).collect(),
                joined_at: m.joined_at.map(|ts| ts.to_rfc3339()),
                premium_since: m.premium_since.map(|ts| ts.to_rfc3339()),
                deaf: m.deaf(),
                mute: m.mute(),
                flags: m.flags.bits(),
                pending: m.pending(),
                permissions: m.permissions.map(|p| format!("{:?}", p)),
                communication_disabled_until: m
                    .communication_disabled_until
                    .map(|ts| ts.to_rfc3339()),
            }),
            data,
            locale: Some(interaction.locale.to_string()),
            guild_locale: interaction.guild_locale.as_ref().map(|l| l.to_string()),
            message_id: Some(interaction.message.id.get().to_string()),
        }
    }
}

#[expose_payload]
pub struct EventModalSubmit {
    interaction_id: String,
    interaction_token: String,
    application_id: String,
    guild_id: Option<String>,
    channel_id: Option<String>,
    user: EventUser,
    member: Option<EventMember>,
    data: serde_json::Value,
    locale: Option<String>,
    guild_locale: Option<String>,
    message_id: Option<String>,
}

impl From<&ModalInteraction> for EventModalSubmit {
    fn from(interaction: &ModalInteraction) -> Self {
        let data = serde_json::to_value(&interaction.data).unwrap_or_default();
        Self {
            interaction_id: interaction.id.get().to_string(),
            interaction_token: interaction.token.to_string(),
            application_id: interaction.application_id.get().to_string(),
            guild_id: interaction.guild_id.map(|g| g.get().to_string()),
            channel_id: Some(interaction.channel_id.get().to_string()),
            user: EventUser::from(&interaction.user),
            member: interaction.member.as_ref().map(|m| EventMember {
                user: EventUser::from(&interaction.user),
                nick: m.nick.as_ref().map(|n| n.to_string()),
                avatar: m.avatar.map(|h| h.to_string()),
                roles: m.roles.iter().map(|r| r.get().to_string()).collect(),
                joined_at: m.joined_at.map(|ts| ts.to_rfc3339()),
                premium_since: m.premium_since.map(|ts| ts.to_rfc3339()),
                deaf: m.deaf(),
                mute: m.mute(),
                flags: m.flags.bits(),
                pending: m.pending(),
                permissions: m.permissions.map(|p| format!("{:?}", p)),
                communication_disabled_until: m
                    .communication_disabled_until
                    .map(|ts| ts.to_rfc3339()),
            }),
            data,
            locale: Some(interaction.locale.to_string()),
            guild_locale: interaction.guild_locale.as_ref().map(|l| l.to_string()),
            message_id: interaction.message.as_ref().map(|m| m.id.get().to_string()),
        }
    }
}

#[expose_payload]
pub struct EventReaction {
    user_id: Option<String>,
    channel_id: String,
    message_id: String,
    guild_id: Option<String>,
    emoji: serde_json::Value,
    burst: bool,
}

impl From<&Reaction> for EventReaction {
    fn from(reaction: &Reaction) -> Self {
        let emoji = serde_json::to_value(&reaction.emoji).unwrap_or_default();
        Self {
            user_id: reaction.user_id.map(|u| u.get().to_string()),
            channel_id: reaction.channel_id.get().to_string(),
            message_id: reaction.message_id.get().to_string(),
            guild_id: reaction.guild_id.map(|g| g.get().to_string()),
            emoji,
            burst: reaction.burst,
        }
    }
}

#[expose_payload]
pub struct EventReactionRemoveAll {
    message_id: String,
    channel_id: String,
    guild_id: Option<String>,
}

impl DiscordHandler {
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
        self.runtime
            .deploy_guild_script(deployment)
            .await
            .map_err(|err| eyre!(err.to_string()))?;
        info!(target: "flora:deployments", guild_id = guild_str, "bootstrapped default script");
        Ok(())
    }
}

impl From<&Ready> for EventReady {
    fn from(ready: &Ready) -> Self {
        Self {
            user: EventUser {
                id: ready.user.id.get().to_string(),
                username: ready.user.name.to_string(),
                discriminator: ready.user.discriminator.map(|d| d.get()),
                bot: ready.user.bot(),
            },
            guild_ids: ready
                .guilds
                .iter()
                .map(|g| g.id.get().to_string())
                .collect(),
        }
    }
}

// Ship a minimal starter bot to new guilds without deployments.
const DEFAULT_GUILD_ENTRY: &str = "src/main.ts";
const DEFAULT_GUILD_SCRIPT: &str = include_str!("../../../example/src/main.ts");
const DEFAULT_GUILD_UTILS_REPLY: &str = include_str!("../../../example/src/utils/reply.ts");
