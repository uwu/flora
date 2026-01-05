use std::{cell::RefCell, rc::Rc, sync::Arc};

use base64::Engine;
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use serde::Deserialize;
use ts_rs::TS;
use serenity::{
    builder::{
        CreateAllowedMentions, CreateAttachment, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
        CreateMessage,
    },
    http::Http,
    model::{
        Color, Timestamp,
        channel::MessageFlags,
        id::{ChannelId, MessageId, RoleId, UserId},
    },
};
use tracing::info;

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase", untagged)]
#[ts(export, export_to = "sdk/src/generated/")]
pub(crate) enum AttachmentInput {
    Url { url: String, filename: Option<String>, description: Option<String> },
    Base64 { data: String, filename: String, description: Option<String> },
}

#[derive(Debug, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "sdk/src/generated/")]
pub(crate) struct EmbedMediaInput {
    url: Option<String>,
}

#[derive(Debug, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "sdk/src/generated/")]
pub(crate) struct EmbedFooterInput {
    text: Option<String>,
    icon_url: Option<String>,
}

#[derive(Debug, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "sdk/src/generated/")]
pub(crate) struct EmbedAuthorInput {
    name: Option<String>,
    url: Option<String>,
    icon_url: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "sdk/src/generated/")]
pub(crate) struct EmbedFieldInput {
    name: String,
    value: String,
    #[serde(default)]
    inline: bool,
}

#[derive(Debug, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "sdk/src/generated/")]
pub(crate) struct EmbedInput {
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    color: Option<u32>,
    timestamp: Option<String>,
    footer: Option<EmbedFooterInput>,
    image: Option<EmbedMediaInput>,
    thumbnail: Option<EmbedMediaInput>,
    author: Option<EmbedAuthorInput>,
    fields: Option<Vec<EmbedFieldInput>>,
}

#[derive(Debug, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "sdk/src/generated/")]
pub(crate) struct AllowedMentionsInput {
    parse: Option<Vec<String>>,
    users: Option<Vec<String>>,
    roles: Option<Vec<String>>,
    #[serde(alias = "repliedUser")]
    replied_user: Option<bool>,
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "sdk/src/generated/")]
pub struct SendMessageArgs {
    #[serde(alias = "channelId")]
    pub channel_id: String,
    pub content: Option<String>,
    pub embeds: Option<Vec<EmbedInput>>,
    pub attachments: Option<Vec<AttachmentInput>>,
    pub tts: Option<bool>,
    #[serde(alias = "allowedMentions")]
    pub allowed_mentions: Option<AllowedMentionsInput>,
    pub flags: Option<u64>,
    #[serde(alias = "messageId")]
    pub message_id: Option<String>,
    #[serde(alias = "replyTo")]
    pub reply_to: Option<String>,
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "sdk/src/generated/")]
pub struct EditMessageArgs {
    #[serde(alias = "channelId")]
    pub channel_id: String,
    #[serde(alias = "messageId")]
    pub message_id: String,
    pub content: Option<String>,
    pub embeds: Option<Vec<EmbedInput>>,
    #[serde(alias = "allowedMentions")]
    pub allowed_mentions: Option<AllowedMentionsInput>,
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
    #[serde] args: SendMessageArgs,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };

    let channel_id_num =
        args.channel_id.parse::<u64>().map_err(|_| JsErrorBox::generic("Invalid channel id"))?;
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

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }

    if let Some(tts) = args.tts {
        message = message.tts(tts);
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds.into_iter().map(build_embed).collect::<Result<Vec<_>, _>>()?;
        has_embeds = !embeds.is_empty();
        message = message.add_embeds(embeds);
    }

    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }

    if let Some(flags) = args.flags {
        message = message.flags(MessageFlags::from_bits_truncate(flags));
    }

    if let Some(message_id_str) = reply_to {
        let message_id =
            message_id_str.parse::<u64>().map_err(|_| JsErrorBox::generic("Invalid message id"))?;
        let reference = MessageId::new(message_id);
        message = message.reference_message((channel_id, reference));
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
    if !has_content && !has_embeds && !has_attachments {
        return Err(JsErrorBox::generic("Message must include content, embeds, or attachments"));
    }

    channel_id
        .send_message(&http, message)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_edit_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: EditMessageArgs,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };

    let channel_id_num =
        args.channel_id.parse::<u64>().map_err(|_| JsErrorBox::generic("Invalid channel id"))?;
    let message_id_num =
        args.message_id.parse::<u64>().map_err(|_| JsErrorBox::generic("Invalid message id"))?;
    let channel_id = ChannelId::new(channel_id_num);
    let message_id = MessageId::new(message_id_num);

    let mut message = serenity::builder::EditMessage::new();
    let mut has_payload = false;

    if let Some(content) = args.content {
        message = message.content(content);
        has_payload = true;
    }

    if let Some(embeds) = args.embeds {
        let embeds = embeds.into_iter().map(build_embed).collect::<Result<Vec<_>, _>>()?;
        message = message.embeds(embeds);
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
        .edit_message(&http, message_id, message)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

pub(crate) fn build_allowed_mentions(input: AllowedMentionsInput) -> CreateAllowedMentions {
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
        let ids = users.into_iter().filter_map(|id| id.parse::<u64>().ok()).map(UserId::new);
        allowed = allowed.users(ids);
    }

    if let Some(roles) = input.roles {
        let ids = roles.into_iter().filter_map(|id| id.parse::<u64>().ok()).map(RoleId::new);
        allowed = allowed.roles(ids);
    }

    if let Some(replied_user) = input.replied_user {
        allowed = allowed.replied_user(replied_user);
    }

    allowed
}

pub(crate) fn build_embed(input: EmbedInput) -> Result<CreateEmbed, JsErrorBox> {
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
    attachment: AttachmentInput,
) -> Result<CreateAttachment, JsErrorBox> {
    match attachment {
        AttachmentInput::Url { url, filename, description } => {
            let mut att = serenity::builder::CreateAttachment::url(http, &url)
                .await
                .map_err(|err| JsErrorBox::generic(err.to_string()))?;

            if let Some(name) = filename {
                att.filename = name;
            }
            if let Some(desc) = description {
                att = att.description(desc);
            }
            Ok(att)
        }
        AttachmentInput::Base64 { data, filename, description } => {
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
