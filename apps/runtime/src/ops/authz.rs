use deno_core::OpState;
use serenity::model::id::{ChannelId, GuildId, ThreadId, WebhookId};

use crate::services::scope_cache::ScopeCache;

use super::FloraError;

pub fn ensure_guild_scope(state: &OpState, guild_id: GuildId) -> Result<(), FloraError> {
    let runtime_guild_id = runtime_guild_id_from_state(state)?;
    if runtime_guild_id != guild_id {
        return Err(FloraError::scope_forbidden(
            "guild is outside runtime scope",
        ));
    }

    Ok(())
}

pub fn runtime_guild_id_from_state(state: &OpState) -> Result<GuildId, FloraError> {
    runtime_guild_id(state)
}

pub async fn ensure_channel_scope(
    runtime_guild_id: GuildId,
    scope_cache: &ScopeCache,
    channel_id: ChannelId,
) -> Result<(), FloraError> {
    let channel_guild_id = scope_cache.resolve_channel(channel_id).await?;
    let Some(channel_guild_id) = channel_guild_id else {
        return Err(FloraError::scope_forbidden(
            "channel is not a guild channel",
        ));
    };

    if channel_guild_id != runtime_guild_id {
        return Err(FloraError::scope_forbidden(
            "channel is outside runtime scope",
        ));
    }

    Ok(())
}

pub async fn ensure_thread_scope(
    runtime_guild_id: GuildId,
    scope_cache: &ScopeCache,
    thread_id: ThreadId,
) -> Result<(), FloraError> {
    ensure_channel_scope(
        runtime_guild_id,
        scope_cache,
        ChannelId::new(thread_id.get()),
    )
    .await
}

pub async fn ensure_webhook_scope(
    runtime_guild_id: GuildId,
    scope_cache: &ScopeCache,
    webhook_id: WebhookId,
) -> Result<(), FloraError> {
    let webhook_guild_id = scope_cache.resolve_webhook(webhook_id).await?;
    let Some(webhook_guild_id) = webhook_guild_id else {
        return Err(FloraError::scope_forbidden(
            "webhook is not owned by a guild",
        ));
    };

    if webhook_guild_id != runtime_guild_id {
        return Err(FloraError::scope_forbidden(
            "webhook is outside runtime scope",
        ));
    }

    Ok(())
}

fn runtime_guild_id(state: &OpState) -> Result<GuildId, FloraError> {
    let runtime_guild_id = state
        .try_borrow::<String>()
        .ok_or_else(|| FloraError::scope_forbidden("guild context not available"))?;
    let Ok(runtime_guild_id) = runtime_guild_id.parse::<u64>() else {
        return Err(FloraError::scope_forbidden("invalid runtime guild id"));
    };
    Ok(GuildId::new(runtime_guild_id))
}
