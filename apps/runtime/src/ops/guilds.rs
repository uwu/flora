use super::authz::ensure_guild_scope;
use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::model::id::{GuildId, RoleId, UserId};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use t0x::T0x;

use crate::services::discord_rest::{DiscordRest, RestRetry};

use super::FloraError;

/// Arguments for operations targeting a user in a guild.
#[expose_input]
pub struct RawGuildUser {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The user's snowflake ID.
    pub user_id: String,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
pub async fn op_kick_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGuildUser,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let user_id = parse_user_id(&args.user_id)?;
    let reason = args.reason;
    let route = format!(
        "DELETE /guilds/{}/members/{}",
        guild_id.get(),
        user_id.get()
    );
    rest.execute(guild_id, route, RestRetry::None, move |http| {
        let reason = reason.clone();
        async move { http.kick_member(guild_id, user_id, reason.as_deref()).await }
    })
    .await?;
    Ok(())
}

/// Arguments for banning a member from a guild.
#[expose_input]
pub struct RawBanMember {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The user's snowflake ID.
    pub user_id: String,
    /// Seconds of message history to delete (0-604800).
    pub delete_message_seconds: Option<u32>,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
pub async fn op_ban_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawBanMember,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let user_id = parse_user_id(&args.user_id)?;
    let delete_seconds = args.delete_message_seconds.unwrap_or(0);
    let reason = args.reason;
    let route = format!("PUT /guilds/{}/bans/{}", guild_id.get(), user_id.get());
    rest.execute(guild_id, route, RestRetry::None, move |http| {
        let reason = reason.clone();
        async move {
            http.ban_user(guild_id, user_id, delete_seconds, reason.as_deref())
                .await
        }
    })
    .await?;
    Ok(())
}

#[op2(async)]
pub async fn op_unban_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGuildUser,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let user_id = parse_user_id(&args.user_id)?;
    let reason = args.reason;
    let route = format!("DELETE /guilds/{}/bans/{}", guild_id.get(), user_id.get());
    rest.execute(guild_id, route, RestRetry::None, move |http| {
        let reason = reason.clone();
        async move { http.remove_ban(guild_id, user_id, reason.as_deref()).await }
    })
    .await?;
    Ok(())
}

/// Arguments for adding or removing a role from a member.
#[expose_input]
pub struct RawMemberRole {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The user's snowflake ID.
    pub user_id: String,
    /// The role's snowflake ID.
    pub role_id: String,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
pub async fn op_add_member_role(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawMemberRole,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let user_id = parse_user_id(&args.user_id)?;
    let role_id = parse_role_id(&args.role_id)?;
    let reason = args.reason;
    let route = format!(
        "PUT /guilds/{}/members/{}/roles/{}",
        guild_id.get(),
        user_id.get(),
        role_id.get()
    );
    rest.execute(guild_id, route, RestRetry::None, move |http| {
        let reason = reason.clone();
        async move {
            http.add_member_role(guild_id, user_id, role_id, reason.as_deref())
                .await
        }
    })
    .await?;
    Ok(())
}

#[op2(async)]
pub async fn op_remove_member_role(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawMemberRole,
) -> Result<(), JsErrorBox> {
    let rest = {
        let state = state.borrow();
        state.borrow::<Arc<DiscordRest>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    {
        let state = state.borrow();
        ensure_guild_scope(&state, guild_id)?;
    }
    let user_id = parse_user_id(&args.user_id)?;
    let role_id = parse_role_id(&args.role_id)?;
    let reason = args.reason;
    let route = format!(
        "DELETE /guilds/{}/members/{}/roles/{}",
        guild_id.get(),
        user_id.get(),
        role_id.get()
    );
    rest.execute(guild_id, route, RestRetry::None, move |http| {
        let reason = reason.clone();
        async move {
            http.remove_member_role(guild_id, user_id, role_id, reason.as_deref())
                .await
        }
    })
    .await?;
    Ok(())
}

/// Arguments for editing a guild member.
#[expose_input]
pub struct RawEditMember {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// The user's snowflake ID.
    pub user_id: String,
    /// JSON payload with fields to update.
    pub payload: serde_json::Value,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_edit_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditMember,
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
    let user_id = parse_user_id(&args.user_id)?;
    let payload = args.payload;
    let reason = args.reason;
    let route = format!("PATCH /guilds/{}/members/{}", guild_id.get(), user_id.get());
    let member = rest
        .execute(guild_id, route, RestRetry::None, move |http| {
            let payload = payload.clone();
            let reason = reason.clone();
            async move {
                http.edit_member(guild_id, user_id, &payload, reason.as_deref())
                    .await
            }
        })
        .await?;
    serde_json::to_value(member).map_err(|err| JsErrorBox::generic(err.to_string()))
}

/// Arguments for editing the current member in a guild.
#[expose_input]
pub struct RawEditCurrentMember {
    /// The guild's snowflake ID.
    pub guild_id: String,
    /// JSON payload with fields to update (nick, avatar, banner, bio).
    pub payload: serde_json::Value,
    /// Audit log reason for this action.
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_edit_current_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditCurrentMember,
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
    let payload = args.payload;
    let reason = args.reason;
    let route = format!("PATCH /guilds/{}/members/@me", guild_id.get());
    let member = rest
        .execute(guild_id, route, RestRetry::None, move |http| {
            let payload = payload.clone();
            let reason = reason.clone();
            async move {
                http.edit_member_me(guild_id, &payload, reason.as_deref())
                    .await
            }
        })
        .await?;
    serde_json::to_value(member).map_err(|err| JsErrorBox::generic(err.to_string()))
}

fn parse_guild_id(value: &str) -> Result<GuildId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("guild_id", "invalid snowflake"));
    };
    Ok(GuildId::new(id))
}

fn parse_user_id(value: &str) -> Result<UserId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("user_id", "invalid snowflake"));
    };
    Ok(UserId::new(id))
}

fn parse_role_id(value: &str) -> Result<RoleId, FloraError> {
    let Ok(id) = value.parse::<u64>() else {
        return Err(FloraError::invalid_input("role_id", "invalid snowflake"));
    };
    Ok(RoleId::new(id))
}
