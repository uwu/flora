use super::components::parse_components;
use super::message::{
    RawAllowedMentions, RawAttachment, RawEmbed, build_allowed_mentions, build_attachment,
    build_embed,
};
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    builder::ExecuteWebhook,
    model::id::{ThreadId, WebhookId},
};
use std::{cell::RefCell, rc::Rc};
use t0x::T0x;

/// Arguments for executing a webhook.
#[expose_input]
pub struct RawExecuteWebhook {
    /// The webhook's snowflake ID.
    pub webhook_id: String,
    /// The webhook token.
    pub token: String,
    /// Whether to wait for message creation and return it.
    pub wait: Option<bool>,
    /// Thread ID to send the message to (if in a forum/thread channel).
    pub thread_id: Option<String>,
    /// Whether to include components (requires specific permissions).
    pub with_components: Option<bool>,
    /// Message content.
    pub content: Option<String>,
    /// Override the webhook's default username.
    pub username: Option<String>,
    /// Override the webhook's default avatar.
    pub avatar_url: Option<String>,
    /// Whether the message should be text-to-speech.
    pub tts: Option<bool>,
    /// Embeds to include.
    pub embeds: Option<Vec<RawEmbed>>,
    /// Attachments to include.
    pub attachments: Option<Vec<RawAttachment>>,
    /// Message components.
    pub components: Option<Vec<serde_json::Value>>,
    /// Allowed mentions configuration.
    pub allowed_mentions: Option<RawAllowedMentions>,
    /// Message flags bitmask.
    pub flags: Option<u64>,
    /// Name for the created thread (forum channels).
    pub thread_name: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_execute_webhook(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawExecuteWebhook,
) -> Result<Option<serde_json::Value>, JsErrorBox> {
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let webhook_id = parse_webhook_id(&args.webhook_id)?;
    let thread_id = match &args.thread_id {
        Some(id) => Some(parse_thread_id(id)?),
        None => None,
    };
    let wait = args.wait.unwrap_or(false);
    let with_components = args.with_components.unwrap_or(false);

    let mut message = ExecuteWebhook::new();
    let mut has_content = false;
    let mut has_embeds = false;
    let mut has_attachments = false;
    let mut has_components = false;

    if let Some(content) = args.content {
        message = message.content(content);
        has_content = true;
    }
    if let Some(username) = args.username {
        message = message.username(username);
    }
    if let Some(avatar_url) = args.avatar_url {
        message = message.avatar_url(avatar_url);
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
        message = message.embeds(embeds);
    }
    if let Some(components) = args.components {
        let components = parse_components(components)?;
        has_components = !components.is_empty();
        message = message.components(components);
    }
    if let Some(mentions) = args.allowed_mentions {
        message = message.allowed_mentions(build_allowed_mentions(mentions));
    }
    if let Some(flags) = args.flags {
        message = message.flags(serenity::model::channel::MessageFlags::from_bits_truncate(
            flags,
        ));
    }
    if let Some(thread_name) = args.thread_name {
        message = message.thread_name(thread_name.into());
    }

    let mut files = Vec::new();
    if let Some(attachments) = args.attachments {
        for attachment in attachments {
            files.push(build_attachment(&http, attachment).await?);
        }
        has_attachments = !files.is_empty();
        message = message.files(files.clone());
    }

    if !has_content && !has_embeds && !has_attachments && !has_components {
        return Err(JsErrorBox::generic(
            "Webhook must include content, embeds, attachments, or components",
        ));
    }

    let result = if with_components {
        http.execute_webhook_with_components(
            webhook_id,
            thread_id,
            &args.token,
            wait,
            files,
            &message,
        )
        .await
    } else {
        http.execute_webhook(webhook_id, thread_id, &args.token, wait, files, &message)
            .await
    }
    .map_err(|err| JsErrorBox::generic(err.to_string()))?;

    match result {
        Some(message) => Ok(Some(
            serde_json::to_value(message).map_err(|err| JsErrorBox::generic(err.to_string()))?,
        )),
        None => Ok(None),
    }
}

/// Arguments for editing a webhook.
#[expose_input]
pub struct RawEditWebhook {
    /// The webhook's snowflake ID.
    pub webhook_id: String,
    /// The webhook token (if using tokenized endpoint).
    pub token: Option<String>,
    /// JSON payload with updated properties.
    pub payload: serde_json::Value,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_edit_webhook(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditWebhook,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let webhook_id = parse_webhook_id(&args.webhook_id)?;
    let webhook = if let Some(token) = args.token {
        http.edit_webhook_with_token(webhook_id, &token, &args.payload, args.reason.as_deref())
            .await
    } else {
        http.edit_webhook(webhook_id, &args.payload, args.reason.as_deref())
            .await
    }
    .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(webhook).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for deleting a webhook.
#[expose_input]
pub struct RawDeleteWebhook {
    /// The webhook's snowflake ID.
    pub webhook_id: String,
    /// The webhook token (if using tokenized endpoint).
    pub token: Option<String>,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
pub async fn op_delete_webhook(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteWebhook,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        super::resolve_http(&state)?
    };
    let webhook_id = parse_webhook_id(&args.webhook_id)?;
    if let Some(token) = args.token {
        http.delete_webhook_with_token(webhook_id, &token, args.reason.as_deref())
            .await
            .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    } else {
        http.delete_webhook(webhook_id, args.reason.as_deref())
            .await
            .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    }
    Ok(())
}

fn parse_webhook_id(value: &str) -> Result<WebhookId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(WebhookId::new)
        .map_err(|_| JsErrorBox::generic("Invalid webhook id"))
}

fn parse_thread_id(value: &str) -> Result<ThreadId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(ThreadId::new)
        .map_err(|_| JsErrorBox::generic("Invalid thread id"))
}
