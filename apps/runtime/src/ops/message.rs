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
use tracing::info;
use ts_rs::TS;
use url::Url;

use super::components::parse_components;

// Note: RawAttachment is an enum, so we keep manual derives
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase", untagged)]
#[ts(export, export_to = "RawAttachment.ts")]
pub enum RawAttachment {
    Url {
        url: String,
        filename: Option<String>,
        description: Option<String>,
    },
    Base64 {
        data: String,
        filename: String,
        description: Option<String>,
    },
}

#[expose_input]
#[derive(Default)]
pub struct RawEmbedMedia {
    url: Option<String>,
}

#[expose_input]
#[derive(Default)]
pub struct RawEmbedFooter {
    text: Option<String>,
    icon_url: Option<String>,
}

#[expose_input]
#[derive(Default)]
pub struct RawEmbedAuthor {
    name: Option<String>,
    url: Option<String>,
    icon_url: Option<String>,
}

#[expose_input]
pub struct RawEmbedField {
    name: String,
    value: String,
    #[serde(default)]
    inline: bool,
}

#[expose_input]
#[derive(Default)]
pub struct RawEmbed {
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    color: Option<u32>,
    timestamp: Option<String>,
    footer: Option<RawEmbedFooter>,
    image: Option<RawEmbedMedia>,
    thumbnail: Option<RawEmbedMedia>,
    author: Option<RawEmbedAuthor>,
    fields: Option<Vec<RawEmbedField>>,
}

#[expose_input]
#[derive(Default)]
pub struct RawAllowedMentions {
    parse: Option<Vec<String>>,
    users: Option<Vec<String>>,
    roles: Option<Vec<String>>,
    replied_user: Option<bool>,
}

#[expose_input]
pub struct RawSendMessage {
    pub channel_id: String,
    pub content: Option<String>,
    pub embeds: Option<Vec<RawEmbed>>,
    pub attachments: Option<Vec<RawAttachment>>,
    pub components: Option<Vec<serde_json::Value>>,
    pub tts: Option<bool>,
    pub allowed_mentions: Option<RawAllowedMentions>,
    pub flags: Option<u64>,
    pub message_id: Option<String>,
    pub reply_to: Option<String>,
}

#[expose_input]
pub struct RawEditMessage {
    pub channel_id: String,
    pub message_id: String,
    pub content: Option<String>,
    pub embeds: Option<Vec<RawEmbed>>,
    pub components: Option<Vec<serde_json::Value>>,
    pub allowed_mentions: Option<RawAllowedMentions>,
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
        state.borrow::<Arc<Http>>().clone()
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
        state.borrow::<Arc<Http>>().clone()
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

#[expose_input]
pub struct RawDeleteMessage {
    pub channel_id: String,
    pub message_id: String,
}

#[op2(async)]
pub async fn op_delete_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteMessage,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
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

#[expose_input]
pub struct RawBulkDeleteMessages {
    pub channel_id: String,
    pub message_ids: Vec<String>,
}

#[op2(async)]
pub async fn op_bulk_delete_messages(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawBulkDeleteMessages,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
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

#[expose_input]
pub struct RawPinMessage {
    pub channel_id: String,
    pub message_id: String,
}

#[op2(async)]
pub async fn op_pin_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawPinMessage,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
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
        state.borrow::<Arc<Http>>().clone()
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

#[expose_input]
pub struct RawCrosspostMessage {
    pub channel_id: String,
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
        state.borrow::<Arc<Http>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let message = channel_id
        .crosspost(&http, message_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub struct RawFetchMessage {
    pub channel_id: String,
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
        state.borrow::<Arc<Http>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let message = http
        .get_message(channel_id.widen(), message_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub struct RawFetchMessages {
    pub channel_id: String,
    pub limit: Option<u8>,
    pub before: Option<String>,
    pub after: Option<String>,
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
        state.borrow::<Arc<Http>>().clone()
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

#[expose_input]
pub struct RawReaction {
    pub channel_id: String,
    pub message_id: String,
    pub emoji: String,
    pub user_id: Option<String>,
}

#[op2(async)]
pub async fn op_add_reaction(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawReaction,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
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
        state.borrow::<Arc<Http>>().clone()
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

#[expose_input]
pub struct RawClearReactions {
    pub channel_id: String,
    pub message_id: String,
    pub emoji: Option<String>,
}

#[op2(async)]
pub async fn op_clear_reactions(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawClearReactions,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
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

pub(crate) fn build_allowed_mentions(
    input: RawAllowedMentions,
) -> CreateAllowedMentions<'static> {
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
