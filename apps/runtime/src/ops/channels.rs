use super::authz::{
    ensure_channel_scope, ensure_guild_scope, ensure_thread_scope, runtime_guild_id_from_state,
};
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::model::id::{ChannelId, GuildId, MessageId, ThreadId, UserId};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use t0x::T0x;

use crate::services::discord_rest::{DiscordRest, RestRetry};

use super::FloraError;

/// Arguments for creating a guild channel.
#[expose_input]
pub struct RawCreateChannel {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// JSON payload with channel properties.
    pub payload: serde_json::Value,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_create_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateChannel,
) -> Result<serde_json::Value, JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let payload = args.payload.clone();
    let reason = args.reason.clone();
    let route = format!("POST /guilds/{}/channels", guild_id.get());
    let channel = rest
        .execute(guild_id, route, RestRetry::None, move |http| {
            let payload = payload.clone();
            let reason = reason.clone();
            async move {
                http.create_channel(guild_id, &payload, reason.as_deref())
                    .await
            }
        })
        .await?;
    serde_json::to_value(channel).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for editing a channel.
#[expose_input]
pub struct RawEditChannel {
    /// The channel's snowflake ID.
    pub channel_id: String,
    /// JSON payload with updated properties.
    pub payload: serde_json::Value,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_edit_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditChannel,
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
    let payload = args.payload.clone();
    let reason = args.reason.clone();
    let route = format!("PATCH /channels/{}", channel_id.get());
    let channel = rest
        .execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let payload = payload.clone();
            let reason = reason.clone();
            async move {
                http.edit_channel(channel_id.widen(), &payload, reason.as_deref())
                    .await
            }
        })
        .await?;
    serde_json::to_value(channel).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for deleting a channel.
#[expose_input]
pub struct RawDeleteChannel {
    /// The channel's snowflake ID.
    pub channel_id: String,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_delete_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteChannel,
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
    let reason = args.reason.clone();
    let route = format!("DELETE /channels/{}", channel_id.get());
    let channel = rest
        .execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let reason = reason.clone();
            async move {
                http.delete_channel(channel_id.widen(), reason.as_deref())
                    .await
            }
        })
        .await?;
    serde_json::to_value(channel).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for creating a thread.
#[expose_input]
pub struct RawCreateThread {
    /// The parent channel's snowflake ID.
    pub channel_id: String,
    /// JSON payload with thread properties.
    pub payload: serde_json::Value,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_create_thread(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateThread,
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
    let payload = args.payload.clone();
    let reason = args.reason.clone();
    let route = format!("POST /channels/{}/threads", channel_id.get());
    let thread = rest
        .execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let payload = payload.clone();
            let reason = reason.clone();
            async move {
                http.create_thread(channel_id, &payload, reason.as_deref())
                    .await
            }
        })
        .await?;
    serde_json::to_value(thread).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for creating a thread from a message.
#[expose_input]
pub struct RawCreateThreadFromMessage {
    /// The parent channel's snowflake ID.
    pub channel_id: String,
    /// The message to start the thread from.
    pub message_id: String,
    /// JSON payload with thread properties.
    pub payload: serde_json::Value,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_create_thread_from_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateThreadFromMessage,
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
    let message_id = parse_message_id(&args.message_id)?;
    let payload = args.payload.clone();
    let reason = args.reason.clone();
    let route = format!(
        "POST /channels/{}/messages/{}/threads",
        channel_id.get(),
        message_id.get()
    );
    let thread = rest
        .execute(runtime_guild_id, route, RestRetry::None, move |http| {
            let payload = payload.clone();
            let reason = reason.clone();
            async move {
                http.create_thread_from_message(channel_id, message_id, &payload, reason.as_deref())
                    .await
            }
        })
        .await?;
    serde_json::to_value(thread).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments containing only a thread ID.
#[expose_input]
pub struct RawThreadId {
    /// The thread's snowflake ID.
    pub thread_id: String,
}

#[op2(async)]
pub async fn op_join_thread(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadId,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_thread_scope(runtime_guild_id, rest.scope_cache(), thread_id).await?;
    let route = format!("PUT /channels/{}/thread-members/@me", thread_id.get());
    rest.execute(
        runtime_guild_id,
        route,
        RestRetry::None,
        move |http| async move { http.join_thread_channel(thread_id).await },
    )
    .await?;
    Ok(())
}

#[op2(async)]
pub async fn op_leave_thread(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadId,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_thread_scope(runtime_guild_id, rest.scope_cache(), thread_id).await?;
    let route = format!("DELETE /channels/{}/thread-members/@me", thread_id.get());
    rest.execute(
        runtime_guild_id,
        route,
        RestRetry::None,
        move |http| async move { http.leave_thread_channel(thread_id).await },
    )
    .await?;
    Ok(())
}

/// Arguments for adding or removing a thread member.
#[expose_input]
pub struct RawThreadMember {
    /// The thread's snowflake ID.
    pub thread_id: String,
    /// The user's snowflake ID.
    pub user_id: String,
}

#[op2(async)]
pub async fn op_add_thread_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadMember,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_thread_scope(runtime_guild_id, rest.scope_cache(), thread_id).await?;
    let user_id = parse_user_id(&args.user_id)?;
    let route = format!(
        "PUT /channels/{}/thread-members/{}",
        thread_id.get(),
        user_id.get()
    );
    rest.execute(
        runtime_guild_id,
        route,
        RestRetry::None,
        move |http| async move { http.add_thread_channel_member(thread_id, user_id).await },
    )
    .await?;
    Ok(())
}

#[op2(async)]
pub async fn op_remove_thread_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadMember,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    let runtime_guild_id = {
        let state = state.borrow();
        runtime_guild_id_from_state(&state)?
    };
    ensure_thread_scope(runtime_guild_id, rest.scope_cache(), thread_id).await?;
    let user_id = parse_user_id(&args.user_id)?;
    let route = format!(
        "DELETE /channels/{}/thread-members/{}",
        thread_id.get(),
        user_id.get()
    );
    rest.execute(
        runtime_guild_id,
        route,
        RestRetry::None,
        move |http| async move { http.remove_thread_channel_member(thread_id, user_id).await },
    )
    .await?;
    Ok(())
}

fn parse_guild_id(value: &str) -> Result<GuildId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("guild_id", "invalid snowflake"));
    };
    Ok(GuildId::new(id))
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

fn parse_thread_id(value: &str) -> Result<ThreadId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("thread_id", "invalid snowflake"));
    };
    Ok(ThreadId::new(id))
}

fn parse_user_id(value: &str) -> Result<UserId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("user_id", "invalid snowflake"));
    };
    Ok(UserId::new(id))
}
