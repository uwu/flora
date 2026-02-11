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

use super::components::parse_components;

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
pub async fn op_send_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawSendMessage,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };

    let channel_id_num = args
        .channel_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid channel id"))?;
    let channel_id = ChannelId::new(channel_id_num);
    let reply_to = args.reply_to.or(args.message_id);
    tracing::info!(
        target: "flora:ops",
        "op_send_message channel={} reply_to={:?}",
        channel_id,
        reply_to
    );

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
        let components = parse_components(components)?;
        has_components = !components.is_empty();
        message = message.components(components);
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
    }

    if let Some(message_id_str) = reply_to {
        let message_id = message_id_str
            .parse::<u64>()
            .map_err(|_| JsErrorBox::generic("Invalid message id"))?;
        let reference = MessageReference::new(MessageReferenceKind::Default, channel_id.widen())
            .message_id(MessageId::new(message_id));
        message = message.reference_message(reference);
    }

    if let Some(attachments) = args.attachments {
        let mut files = Vec::with_capacity(attachments.len());
        for attachment in attachments {
            files.push(build_attachment(&http, attachment).await?);
        }
        has_attachments = !files.is_empty();
        message = message.add_files(files);
    }

    // Fail early if we ended up with an empty payload.
    if !has_content && !has_embeds && !has_attachments && !has_components {
        return Err(JsErrorBox::generic(
            "Message must include content, embeds, attachments, or components",
        ));
    }

    channel_id
        .widen()
        .send_message(&http, message)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_edit_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditMessage,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };

    let channel_id_num = args
        .channel_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid channel id"))?;
    let message_id_num = args
        .message_id
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid message id"))?;
    let channel_id = ChannelId::new(channel_id_num);
    let message_id = MessageId::new(message_id_num);

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
        let components = parse_components(components)?;
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
        return Err(JsErrorBox::generic(
            "Message edit must include content, embeds, flags, or allowed mentions",
        ));
    }

    channel_id
        .widen()
        .edit_message(&http, message_id, message)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    channel_id
        .widen()
        .delete_message(&http, message_id, None)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_ids = args
        .message_ids
        .into_iter()
        .map(|id| parse_message_id(&id))
        .collect::<Result<Vec<_>, _>>()?;
    channel_id
        .widen()
        .delete_messages(&http, &message_ids, None)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    channel_id
        .widen()
        .pin(&http, message_id, None)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_unpin_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawPinMessage,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    channel_id
        .widen()
        .unpin(&http, message_id, None)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let message = channel_id
        .crosspost(&http, message_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let message = http
        .get_message(channel_id.widen(), message_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
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
    let messages = channel_id
        .widen()
        .messages(&http, builder)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    messages
        .into_iter()
        .map(|msg| serde_json::to_value(msg).map_err(|err| JsErrorBox::generic(err.to_string())))
        .collect()
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let reaction = parse_reaction(&args.emoji)?;
    channel_id
        .widen()
        .create_reaction(&http, message_id, reaction)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_remove_reaction(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawReaction,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let reaction = parse_reaction(&args.emoji)?;
    let user_id = if let Some(id) = args.user_id {
        Some(UserId::new(
            id.parse::<u64>()
                .map_err(|_| JsErrorBox::generic("Invalid user id"))?,
        ))
    } else {
        None
    };
    channel_id
        .widen()
        .delete_reaction(&http, message_id, user_id, reaction)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
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
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    if let Some(emoji) = args.emoji {
        let reaction = parse_reaction(&emoji)?;
        channel_id
            .widen()
            .delete_reaction_emoji(&http, message_id, reaction)
            .await
            .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    } else {
        channel_id
            .widen()
            .delete_reactions(&http, message_id)
            .await
            .map_err(|err| JsErrorBox::generic(err.to_string()))?;
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

pub(crate) fn build_embed(input: RawEmbed) -> Result<CreateEmbed<'static>, JsErrorBox> {
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
            .map_err(|_| JsErrorBox::generic("Invalid embed timestamp"))?;
        embed = embed.timestamp(parsed);
    }

    if let Some(footer) = input.footer {
        if let Some(text) = footer.text {
            let mut footer_builder = CreateEmbedFooter::new(text);
            if let Some(icon) = footer.icon_url {
                footer_builder = footer_builder.icon_url(icon);
            }
            embed = embed.footer(footer_builder);
        }
    }

    if let Some(image) = input.image {
        if let Some(url) = image.url {
            embed = embed.image(url);
        }
    }

    if let Some(thumbnail) = input.thumbnail {
        if let Some(url) = thumbnail.url {
            embed = embed.thumbnail(url);
        }
    }

    if let Some(author) = input.author {
        if let Some(name) = author.name {
            let mut author_builder = CreateEmbedAuthor::new(name);
            if let Some(url) = author.url {
                author_builder = author_builder.url(url);
            }
            if let Some(icon) = author.icon_url {
                author_builder = author_builder.icon_url(icon);
            }
            embed = embed.author(author_builder);
        }
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
) -> Result<CreateAttachment<'static>, JsErrorBox> {
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
                        parsed
                            .path_segments()
                            .and_then(|segments| segments.last().map(|name| name.to_string()))
                    })
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| "attachment".to_string())
            });
            let mut att = serenity::builder::CreateAttachment::url(http, &url, resolved_name)
                .await
                .map_err(|err| JsErrorBox::generic(err.to_string()))?;

            if let Some(name) = filename {
                att.filename = name.into();
            }
            if let Some(desc) = description {
                att = att.description(desc);
            }
            Ok(att)
        }
        RawAttachment::Base64 {
            data,
            filename,
            description,
        } => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .map_err(|_| JsErrorBox::generic("Invalid base64 attachment data"))?;
            let mut att = CreateAttachment::bytes(bytes, filename);
            if let Some(desc) = description {
                att = att.description(desc);
            }
            Ok(att)
        }
    }
}

fn parse_channel_id(value: &str) -> Result<ChannelId, JsErrorBox> {
    let id = value
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid channel id"))?;
    Ok(ChannelId::new(id))
}

fn parse_message_id(value: &str) -> Result<MessageId, JsErrorBox> {
    let id = value
        .parse::<u64>()
        .map_err(|_| JsErrorBox::generic("Invalid message id"))?;
    Ok(MessageId::new(id))
}

fn parse_reaction(value: &str) -> Result<serenity::model::channel::ReactionType, JsErrorBox> {
    serenity::model::channel::ReactionType::try_from(value)
        .map_err(|_| JsErrorBox::generic("Invalid reaction emoji"))
}
