use super::message::{
    RawAllowedMentions, RawAttachment, RawEmbed, build_allowed_mentions, build_attachment,
    build_embed,
};
use super::{
    authz::{ensure_thread_scope, ensure_webhook_scope, runtime_guild_id_from_state},
    components::parse_components,
};
use crate::services::discord_rest::{DiscordRest, RestRetry};
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    builder::ExecuteWebhook,
    model::id::{ThreadId, WebhookId},
};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use t0x::T0x;

use super::FloraError;

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
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let webhook_id = parse_webhook_id(&args.webhook_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_webhook_scope(runtime_guild_id, rest.scope_cache(), webhook_id).await?;
    let thread_id = match &args.thread_id {
        Some(id) => Some(parse_thread_id(id)?),
        None => None,
    };
    if let Some(thread_id) = thread_id {
        ensure_thread_scope(runtime_guild_id, rest.scope_cache(), thread_id).await?;
    }
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
        let components = parse_components(components)
            .map_err(|err| FloraError::invalid_input("components", err.to_string()))?;
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
            files.push(build_attachment(rest.http(), attachment).await?);
        }
        has_attachments = !files.is_empty();
        message = message.files(files.clone());
    }

    if !has_content && !has_embeds && !has_attachments && !has_components {
        return Err(FloraError::invalid_input(
            "payload",
            "webhook must include content, embeds, attachments, or components",
        )
        .into());
    }

    let token = args.token;
    let route = format!("POST /webhooks/{}", webhook_id.get());
    let result = rest
        .execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let token = token.clone();
            let files = files.clone();
            let message = message.clone();
            async move {
                if with_components {
                    http.execute_webhook_with_components(
                        webhook_id, thread_id, &token, wait, files, &message,
                    )
                    .await
                } else {
                    http.execute_webhook(webhook_id, thread_id, &token, wait, files, &message)
                        .await
                }
            }
        })
        .await?;

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
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let webhook_id = parse_webhook_id(&args.webhook_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_webhook_scope(runtime_guild_id, rest.scope_cache(), webhook_id).await?;
    let token = args.token;
    let payload = args.payload;
    let reason = args.reason;
    let route = if token.is_some() {
        format!("PATCH /webhooks/{}/token", webhook_id.get())
    } else {
        format!("PATCH /webhooks/{}", webhook_id.get())
    };
    let webhook = rest
        .execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let token = token.clone();
            let payload = payload.clone();
            let reason = reason.clone();
            async move {
                if let Some(token) = token {
                    http.edit_webhook_with_token(webhook_id, &token, &payload, reason.as_deref())
                        .await
                } else {
                    http.edit_webhook(webhook_id, &payload, reason.as_deref())
                        .await
                }
            }
        })
        .await?;
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
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let webhook_id = parse_webhook_id(&args.webhook_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_webhook_scope(runtime_guild_id, rest.scope_cache(), webhook_id).await?;
    let token = args.token;
    let reason = args.reason;
    let route = if token.is_some() {
        format!("DELETE /webhooks/{}/token", webhook_id.get())
    } else {
        format!("DELETE /webhooks/{}", webhook_id.get())
    };
    rest.execute(runtime_guild_id, route, RestRetry::None, move |http| {
        let token = token.clone();
        let reason = reason.clone();
        async move {
            if let Some(token) = token {
                http.delete_webhook_with_token(webhook_id, &token, reason.as_deref())
                    .await
            } else {
                http.delete_webhook(webhook_id, reason.as_deref()).await
            }
        }
    })
    .await?;
    Ok(())
}

fn parse_webhook_id(value: &str) -> Result<WebhookId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("webhook_id", "invalid snowflake"));
    };
    Ok(WebhookId::new(id))
}

fn parse_thread_id(value: &str) -> Result<ThreadId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("thread_id", "invalid snowflake"));
    };
    Ok(ThreadId::new(id))
}
