use deno_core::OpState;
use deno_error::JsErrorBox;
use serenity::{
    http::Http,
    model::{
        channel::Channel,
        id::{ChannelId, GuildId, ThreadId, WebhookId},
    },
};

pub fn ensure_guild_scope(state: &OpState, guild_id: GuildId) -> Result<(), JsErrorBox> {
    let runtime_guild_id = runtime_guild_id_from_state(state)?;
    if runtime_guild_id != guild_id {
        return Err(JsErrorBox::generic(
            "Forbidden: guild is outside runtime scope",
        ));
    }

    Ok(())
}

pub fn runtime_guild_id_from_state(state: &OpState) -> Result<GuildId, JsErrorBox> {
    runtime_guild_id(state)
}

pub async fn ensure_channel_scope(
    runtime_guild_id: GuildId,
    http: &Http,
    channel_id: ChannelId,
) -> Result<(), JsErrorBox> {
    let channel = http
        .get_channel(channel_id.widen())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;

    let Some(channel_guild_id) = channel_guild_id(channel) else {
        return Err(JsErrorBox::generic(
            "Forbidden: channel is not a guild channel",
        ));
    };

    if channel_guild_id != runtime_guild_id {
        return Err(JsErrorBox::generic(
            "Forbidden: channel is outside runtime scope",
        ));
    }

    Ok(())
}

pub async fn ensure_thread_scope(
    runtime_guild_id: GuildId,
    http: &Http,
    thread_id: ThreadId,
) -> Result<(), JsErrorBox> {
    ensure_channel_scope(runtime_guild_id, http, ChannelId::new(thread_id.get())).await
}

pub async fn ensure_webhook_scope(
    runtime_guild_id: GuildId,
    http: &Http,
    webhook_id: WebhookId,
) -> Result<(), JsErrorBox> {
    let webhook = http
        .get_webhook(webhook_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;

    let Some(webhook_guild_id) = webhook.guild_id else {
        return Err(JsErrorBox::generic(
            "Forbidden: webhook is not owned by a guild",
        ));
    };

    if webhook_guild_id != runtime_guild_id {
        return Err(JsErrorBox::generic(
            "Forbidden: webhook is outside runtime scope",
        ));
    }

    Ok(())
}

fn runtime_guild_id(state: &OpState) -> Result<GuildId, JsErrorBox> {
    let runtime_guild_id = state
        .try_borrow::<String>()
        .ok_or_else(|| JsErrorBox::generic("guild context not available"))?;
    runtime_guild_id
        .parse::<u64>()
        .map(GuildId::new)
        .map_err(|_| JsErrorBox::generic("invalid runtime guild id"))
}

fn channel_guild_id(channel: Channel) -> Option<GuildId> {
    channel.guild_id()
}
