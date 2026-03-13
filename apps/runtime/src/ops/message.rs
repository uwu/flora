use base64::Engine;
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serde::Deserialize;
use serenity::{
    builder::{
        CreateAllowedMentions, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
        CreateMessage, GetMessages,
    },
    http::Http,
    model::{
        Color, Timestamp,
        channel::{MessageFlags, MessageReference, MessageReferenceKind},
        id::{ChannelId, MessageId, RoleId, UserId},
    },
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use t0x::T0x;
use tracing::info;
use url::Url;

use crate::services::discord_rest::{DiscordRest, RestRetry};

use super::{
    FloraError,
    authz::{ensure_channel_scope, runtime_guild_id_from_state},
    components::parse_components,
};

/// Attachment to include in a message (either URL or base64-encoded data).
#[derive(Debug, Deserialize, T0x)]
pub enum RawAttachment {
    /// Attachment from a URL.
    Url {
        /// The URL to fetch the attachment from.
        url: String,
        /// Override filename for the attachment.
        filename: Option<String>,
        /// Alt text / description for the attachment.
        description: Option<String>,
    },
    /// Attachment from base64-encoded data.
    Base64 {
        /// Base64-encoded file data.
        data: String,
        /// Filename for the attachment.
        filename: String,
        /// Alt text / description for the attachment.
        description: Option<String>,
    },
}

/// Media (image/video) for an embed.
#[expose_input]
#[derive(Default)]
pub struct RawEmbedMedia {
    /// URL of the media.
    url: Option<String>,
}

/// Footer for an embed.
#[expose_input]
#[derive(Default)]
pub struct RawEmbedFooter {
    /// Footer text.
    text: Option<String>,
    /// URL of the footer icon.
    icon_url: Option<String>,
}

/// Author for an embed.
#[expose_input]
#[derive(Default)]
pub struct RawEmbedAuthor {
    /// Author name.
    name: Option<String>,
    /// URL when clicking the author name.
    url: Option<String>,
    /// URL of the author icon.
    icon_url: Option<String>,
}

/// A field in an embed.
#[expose_input]
pub struct RawEmbedField {
    /// Field name (title).
    name: String,
    /// Field value (content).
    value: String,
    /// Whether to display inline with other fields.
    #[serde(default)]
    inline: bool,
}

/// A rich embed for a message.
///
/// [Discord docs](https://discord.com/developers/docs/resources/message#embed-object).
#[expose_input]
#[derive(Default)]
pub struct RawEmbed {
    /// Embed title.
    title: Option<String>,
    /// Embed description.
    description: Option<String>,
    /// URL when clicking the title.
    url: Option<String>,
    /// Color code (decimal integer).
    color: Option<u32>,
    /// ISO8601 timestamp for the embed.
    timestamp: Option<String>,
    /// Footer information.
    footer: Option<RawEmbedFooter>,
    /// Image to display.
    image: Option<RawEmbedMedia>,
    /// Thumbnail to display.
    thumbnail: Option<RawEmbedMedia>,
    /// Author information.
    author: Option<RawEmbedAuthor>,
    /// Fields to display.
    fields: Option<Vec<RawEmbedField>>,
}

/// Configuration for which mentions are allowed.
///
/// [Discord docs](https://discord.com/developers/docs/resources/message#allowed-mentions-object).
#[expose_input]
#[derive(Default)]
pub struct RawAllowedMentions {
    /// Types of mentions to parse ("everyone", "users", "roles").
    parse: Option<Vec<String>>,
    /// Specific user IDs allowed to be mentioned.
    users: Option<Vec<String>>,
    /// Specific role IDs allowed to be mentioned.
    roles: Option<Vec<String>>,
    /// Whether to mention the user being replied to.
    replied_user: Option<bool>,
}

/// Arguments for sending a message.
#[expose_input]
pub struct RawSendMessage {
    /// The channel to send the message to.
    pub channel_id: String,
    /// Message content.
    pub content: Option<String>,
    /// Embeds to include.
    pub embeds: Option<Vec<RawEmbed>>,
    /// Attachments to include.
    pub attachments: Option<Vec<RawAttachment>>,
    /// Message components.
    pub components: Option<Vec<serde_json::Value>>,
    /// Whether the message should be text-to-speech.
    pub tts: Option<bool>,
    /// Allowed mentions configuration.
    pub allowed_mentions: Option<RawAllowedMentions>,
    /// Message flags bitmask.
    pub flags: Option<u64>,
    /// Deprecated: use reply_to instead.
    pub message_id: Option<String>,
    /// Message ID to reply to.
    pub reply_to: Option<String>,
}

/// Arguments for editing a message.
#[expose_input]
pub struct RawEditMessage {
    /// The channel containing the message.
    pub channel_id: String,
    /// The message to edit.
    pub message_id: String,
    /// New message content.
    pub content: Option<String>,
    /// Embeds to include.
    pub embeds: Option<Vec<RawEmbed>>,
    /// Message components.
    pub components: Option<Vec<serde_json::Value>>,
    /// Allowed mentions configuration.
    pub allowed_mentions: Option<RawAllowedMentions>,
    /// Message flags bitmask.
    pub flags: Option<u64>,
}

#[op2]
pub fn op_log(state: &mut OpState, #[serde] args: Vec<serde_json::Value>) {
    let guild_id = state.try_borrow::<String>().cloned();

    let text = args
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => s,
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ");

    // Send to log sink for SSE streaming
    crate::log_sink::log_js(tracing::Level::INFO, guild_id.clone(), text.clone());

    info!(
        target: "flora:js",
        guild_id = guild_id.as_deref().unwrap_or("default"),
        "{}",
        text
    );
}

#[op2(async)]
#[serde]
pub async fn op_send_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawSendMessage,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };

    let channel_id = parse_channel_id(&args.channel_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let reply_to = args.reply_to.or(args.message_id);
    info!(target: "flora:ops", "op_send_message channel={} reply_to={:?}", channel_id, reply_to);

    let mut message = CreateMessage::new();
    let mut has_content = false;
    let mut has_embeds = false;
    let mut has_attachments = false;
    let mut has_components = false;

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }

    if let Some(tts) = args.tts {
        message = message.tts(tts);
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds
            .into_iter()
            .map(build_embed)
            .collect::<Result<Vec<_>, _>>()?;
        has_embeds = !embeds.is_empty();
        message = message.add_embeds(embeds);
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }

    if let Some(components) = args.components {
        let components = parse_components(components)
            .map_err(|err| FloraError::invalid_input("components", err.to_string()))?;
        has_components = !components.is_empty();
        message = message.components(components);
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
    }

    if let Some(message_id) = reply_to {
        let message_id = parse_message_id(&message_id)?;
        let reference = MessageReference::new(MessageReferenceKind::Default, channel_id.widen())
            .message_id(message_id);
        message = message.reference_message(reference);
    }

    if let Some(attachments) = args.attachments {
        let mut files = Vec::with_capacity(attachments.len());
        for attachment in attachments {
            files.push(build_attachment(rest.http(), attachment).await?);
        }
        has_attachments = !files.is_empty();
        message = message.add_files(files);
    }

    if !has_content && !has_embeds && !has_attachments && !has_components {
        return Err(FloraError::invalid_input(
            "payload",
            "message must include content, embeds, attachments, or components",
        )
        .into());
    }

    let route = format!("POST /channels/{}/messages", channel_id.get());
    let created = rest
        .execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let message = message.clone();
            async move { channel_id.widen().send_message(&http, message).await }
        })
        .await?;

    to_json_value(created).map_err(Into::into)
}

#[op2(async)]
pub async fn op_edit_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditMessage,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };

    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let mut message = serenity::builder::EditMessage::new();
    let mut has_payload = false;

    if let Some(content) = args.content {
        message = message.content(content);
        has_payload = true;
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds
            .into_iter()
            .map(build_embed)
            .collect::<Result<Vec<_>, _>>()?;
        message = message.embeds(embeds);
        has_payload = true;
    }

    if let Some(components) = args.components {
        let components = parse_components(components)
            .map_err(|err| FloraError::invalid_input("components", err.to_string()))?;
        message = message.components(components);
        has_payload = true;
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
        has_payload = true;
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
        has_payload = true;
    }

    if !has_payload {
        return Err(FloraError::invalid_input(
            "payload",
            "message edit must include content, embeds, flags, or allowed mentions",
        )
        .into());
    }

    let route = format!(
        "PATCH /channels/{}/messages/{}",
        channel_id.get(),
        message_id.get()
    );
    rest.execute(runtime_guild_id, route, RestRetry::None, move |http| {
        let message = message.clone();
        async move {
            channel_id
                .widen()
                .edit_message(&http, message_id, message)
                .await
        }
    })
    .await?;

    Ok(())
}

/// Arguments for deleting a message.
#[expose_input]
pub struct RawDeleteMessage {
    /// The channel containing the message.
    pub channel_id: String,
    /// The message to delete.
    pub message_id: String,
}

#[op2(async)]
pub async fn op_delete_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteMessage,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let route = format!(
        "DELETE /channels/{}/messages/{}",
        channel_id.get(),
        message_id.get()
    );
    rest.execute(
        runtime_guild_id,
        route,
        RestRetry::None,
        move |http| async move {
            channel_id
                .widen()
                .delete_message(&http, message_id, None)
                .await
        },
    )
    .await?;

    Ok(())
}

/// Arguments for bulk deleting messages.
#[expose_input]
pub struct RawBulkDeleteMessages {
    /// The channel containing the messages.
    pub channel_id: String,
    /// IDs of messages to delete (2-100).
    pub message_ids: Vec<String>,
}

#[op2(async)]
pub async fn op_bulk_delete_messages(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawBulkDeleteMessages,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let mut message_ids = Vec::with_capacity(args.message_ids.len());
    for id in args.message_ids {
        message_ids.push(parse_message_id(&id)?);
    }

    let route = format!("POST /channels/{}/messages/bulk-delete", channel_id.get());
    rest.execute(runtime_guild_id, route, RestRetry::None, move |http| {
        let message_ids = message_ids.clone();
        async move {
            channel_id
                .widen()
                .delete_messages(&http, &message_ids, None)
                .await
        }
    })
    .await?;

    Ok(())
}

/// Arguments for pinning or unpinning a message.
#[expose_input]
pub struct RawPinMessage {
    /// The channel containing the message.
    pub channel_id: String,
    /// The message to pin/unpin.
    pub message_id: String,
}

#[op2(async)]
pub async fn op_pin_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawPinMessage,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let route = format!(
        "PUT /channels/{}/pins/{}",
        channel_id.get(),
        message_id.get()
    );
    rest.execute(
        runtime_guild_id,
        route,
        RestRetry::None,
        move |http| async move { channel_id.widen().pin(&http, message_id, None).await },
    )
    .await?;

    Ok(())
}

#[op2(async)]
pub async fn op_unpin_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawPinMessage,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let route = format!(
        "DELETE /channels/{}/pins/{}",
        channel_id.get(),
        message_id.get()
    );
    rest.execute(
        runtime_guild_id,
        route,
        RestRetry::None,
        move |http| async move { channel_id.widen().unpin(&http, message_id, None).await },
    )
    .await?;

    Ok(())
}

/// Arguments for crossposting a message to followers.
#[expose_input]
pub struct RawCrosspostMessage {
    /// The announcement channel containing the message.
    pub channel_id: String,
    /// The message to crosspost.
    pub message_id: String,
}

#[op2(async)]
#[serde]
pub async fn op_crosspost_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCrosspostMessage,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let route = format!(
        "POST /channels/{}/messages/{}/crosspost",
        channel_id.get(),
        message_id.get()
    );
    let message = rest
        .execute(
            runtime_guild_id,
            route,
            RestRetry::None,
            move |http| async move { channel_id.crosspost(&http, message_id).await },
        )
        .await?;

    to_json_value(message).map_err(Into::into)
}

/// Arguments for fetching a single message.
#[expose_input]
pub struct RawFetchMessage {
    /// The channel containing the message.
    pub channel_id: String,
    /// The message to fetch.
    pub message_id: String,
}

#[op2(async)]
#[serde]
pub async fn op_fetch_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawFetchMessage,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let route = format!(
        "GET /channels/{}/messages/{}",
        channel_id.get(),
        message_id.get()
    );
    let message = rest
        .execute(
            runtime_guild_id,
            route,
            RestRetry::ReadOnly,
            move |http| async move { http.get_message(channel_id.widen(), message_id).await },
        )
        .await?;

    to_json_value(message).map_err(Into::into)
}

/// Arguments for fetching multiple messages from a channel.
#[expose_input]
pub struct RawFetchMessages {
    /// The channel to fetch messages from.
    pub channel_id: String,
    /// Maximum number of messages to return (1-100).
    pub limit: Option<u8>,
    /// Fetch messages before this message ID.
    pub before: Option<String>,
    /// Fetch messages after this message ID.
    pub after: Option<String>,
    /// Fetch messages around this message ID.
    pub around: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_fetch_messages(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawFetchMessages,
) -> Result<Vec<serde_json::Value>, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let mut builder = GetMessages::new();
    if let Some(limit) = args.limit {
        builder = builder.limit(limit);
    }
    if let Some(before) = args.before {
        builder = builder.before(parse_message_id(&before)?);
    }
    if let Some(after) = args.after {
        builder = builder.after(parse_message_id(&after)?);
    }
    if let Some(around) = args.around {
        builder = builder.around(parse_message_id(&around)?);
    }

    let route = format!("GET /channels/{}/messages", channel_id.get());
    let messages = rest
        .execute(
            runtime_guild_id,
            route,
            RestRetry::ReadOnly,
            move |http| async move { channel_id.widen().messages(&http, builder).await },
        )
        .await?;

    to_json_values(messages).map_err(Into::into)
}

/// Arguments for adding or removing a reaction.
#[expose_input]
pub struct RawReaction {
    /// The channel containing the message.
    pub channel_id: String,
    /// The message to react to.
    pub message_id: String,
    /// The emoji to use (unicode or custom format).
    pub emoji: String,
    /// User ID to remove reaction from (for remove only).
    pub user_id: Option<String>,
}

#[op2(async)]
pub async fn op_add_reaction(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawReaction,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let reaction = parse_reaction(&args.emoji)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let route = format!(
        "PUT /channels/{}/messages/{}/reactions/@me",
        channel_id.get(),
        message_id.get()
    );
    rest.execute(runtime_guild_id, route, RestRetry::None, move |http| {
        let reaction = reaction.clone();
        async move {
            channel_id
                .widen()
                .create_reaction(&http, message_id, reaction)
                .await
        }
    })
    .await?;

    Ok(())
}

#[op2(async)]
pub async fn op_remove_reaction(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawReaction,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let reaction = parse_reaction(&args.emoji)?;
    let user_id = if let Some(user_id) = args.user_id {
        Some(parse_user_id(&user_id)?)
    } else {
        None
    };
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    let route = format!(
        "DELETE /channels/{}/messages/{}/reactions",
        channel_id.get(),
        message_id.get()
    );
    rest.execute(runtime_guild_id, route, RestRetry::None, move |http| {
        let reaction = reaction.clone();
        async move {
            channel_id
                .widen()
                .delete_reaction(&http, message_id, user_id, reaction)
                .await
        }
    })
    .await?;

    Ok(())
}

/// Arguments for clearing reactions from a message.
#[expose_input]
pub struct RawClearReactions {
    /// The channel containing the message.
    pub channel_id: String,
    /// The message to clear reactions from.
    pub message_id: String,
    /// Specific emoji to clear (if omitted, clears all reactions).
    pub emoji: Option<String>,
}

#[op2(async)]
pub async fn op_clear_reactions(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawClearReactions,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_channel_scope(runtime_guild_id, rest.scope_cache(), channel_id).await?;

    if let Some(emoji) = args.emoji {
        let reaction = parse_reaction(&emoji)?;
        let route = format!(
            "DELETE /channels/{}/messages/{}/reactions/emoji",
            channel_id.get(),
            message_id.get()
        );
        rest.execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let reaction = reaction.clone();
            async move {
                channel_id
                    .widen()
                    .delete_reaction_emoji(&http, message_id, reaction)
                    .await
            }
        })
        .await?;
    } else {
        let route = format!(
            "DELETE /channels/{}/messages/{}/reactions",
            channel_id.get(),
            message_id.get()
        );
        rest.execute(
            runtime_guild_id,
            route,
            RestRetry::None,
            move |http| async move { channel_id.widen().delete_reactions(&http, message_id).await },
        )
        .await?;
    }

    Ok(())
}

pub(crate) fn build_allowed_mentions(input: RawAllowedMentions) -> CreateAllowedMentions<'static> {
    let mut allowed = CreateAllowedMentions::new();

    if let Some(parse) = input.parse {
        for entry in parse {
            match entry.as_str() {
                "everyone" => allowed = allowed.everyone(true),
                "users" => allowed = allowed.all_users(true),
                "roles" => allowed = allowed.all_roles(true),
                _ => {}
            }
        }
    }

    if let Some(users) = input.users {
        let ids: Vec<UserId> = users
            .into_iter()
            .filter_map(|id| id.parse::<u64>().ok())
            .map(UserId::new)
            .collect();
        allowed = allowed.users(ids);
    }

    if let Some(roles) = input.roles {
        let ids: Vec<RoleId> = roles
            .into_iter()
            .filter_map(|id| id.parse::<u64>().ok())
            .map(RoleId::new)
            .collect();
        allowed = allowed.roles(ids);
    }

    if let Some(replied_user) = input.replied_user {
        allowed = allowed.replied_user(replied_user);
    }

    allowed
}

pub(crate) fn build_embed(input: RawEmbed) -> Result<CreateEmbed<'static>, FloraError> {
    let mut embed = CreateEmbed::new();

    if let Some(title) = input.title {
        embed = embed.title(title);
    }

    if let Some(description) = input.description {
        embed = embed.description(description);
    }

    if let Some(url) = input.url {
        embed = embed.url(url);
    }

    if let Some(color) = input.color {
        embed = embed.color(Color::new(color));
    }

    if let Some(timestamp) = input.timestamp {
        let parsed = timestamp
            .parse::<Timestamp>()
            .map_err(|_| FloraError::invalid_input("embeds[].timestamp", "invalid timestamp"))?;
        embed = embed.timestamp(parsed);
    }

    if let Some(footer) = input.footer
        && let Some(text) = footer.text
    {
        let mut footer_builder = CreateEmbedFooter::new(text);
        if let Some(icon) = footer.icon_url {
            footer_builder = footer_builder.icon_url(icon);
        }
        embed = embed.footer(footer_builder);
    }

    if let Some(image) = input.image
        && let Some(url) = image.url
    {
        embed = embed.image(url);
    }

    if let Some(thumbnail) = input.thumbnail
        && let Some(url) = thumbnail.url
    {
        embed = embed.thumbnail(url);
    }

    if let Some(author) = input.author
        && let Some(name) = author.name
    {
        let mut author_builder = CreateEmbedAuthor::new(name);
        if let Some(url) = author.url {
            author_builder = author_builder.url(url);
        }
        if let Some(icon) = author.icon_url {
            author_builder = author_builder.icon_url(icon);
        }
        embed = embed.author(author_builder);
    }

    if let Some(fields) = input.fields {
        for field in fields {
            embed = embed.field(field.name, field.value, field.inline);
        }
    }

    Ok(embed)
}

pub(crate) async fn build_attachment(
    http: &Arc<Http>,
    attachment: RawAttachment,
) -> Result<CreateAttachment<'static>, FloraError> {
    match attachment {
        RawAttachment::Url {
            url,
            filename,
            description,
        } => {
            let resolved_name = filename.clone().unwrap_or_else(|| {
                Url::parse(&url)
                    .ok()
                    .and_then(|parsed| {
                        parsed.path_segments().and_then(|mut segments| {
                            segments.next_back().map(|name| name.to_string())
                        })
                    })
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| "attachment".to_string())
            });
            let mut attachment =
                serenity::builder::CreateAttachment::url(http, &url, resolved_name)
                    .await
                    .map_err(|err| FloraError::invalid_input("attachments", err.to_string()))?;

            if let Some(filename) = filename {
                attachment.filename = filename.into();
            }
            if let Some(description) = description {
                attachment = attachment.description(description);
            }

            Ok(attachment)
        }
        RawAttachment::Base64 {
            data,
            filename,
            description,
        } => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .map_err(|_| {
                    FloraError::invalid_input("attachments[].data", "invalid base64 data")
                })?;
            let mut attachment = CreateAttachment::bytes(bytes, filename);
            if let Some(description) = description {
                attachment = attachment.description(description);
            }
            Ok(attachment)
        }
    }
}

fn parse_channel_id(value: &str) -> Result<ChannelId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("channel_id", "invalid snowflake"));
    };
    Ok(ChannelId::new(id))
}

fn parse_message_id(value: &str) -> Result<MessageId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("message_id", "invalid snowflake"));
    };
    Ok(MessageId::new(id))
}

fn parse_user_id(value: &str) -> Result<UserId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("user_id", "invalid snowflake"));
    };
    Ok(UserId::new(id))
}

fn parse_reaction(value: &str) -> Result<serenity::model::channel::ReactionType, FloraError> {
    serenity::model::channel::ReactionType::try_from(value)
        .map_err(|_| FloraError::invalid_input("emoji", "invalid reaction emoji"))
}

fn to_json_value<T: serde::Serialize>(value: T) -> Result<serde_json::Value, FloraError> {
    serde_json::to_value(value).map_err(|err| {
        FloraError::discord_http(500, 0, format!("failed to serialize response: {err}"))
    })
}

fn to_json_values<T: serde::Serialize>(
    values: Vec<T>,
) -> Result<Vec<serde_json::Value>, FloraError> {
    let mut json_values = Vec::with_capacity(values.len());
    for value in values {
        json_values.push(to_json_value(value)?);
    }
    Ok(json_values)
}
