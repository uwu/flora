use std::{cell::RefCell, rc::Rc, sync::Arc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    http::Http,
    model::id::{GuildId, RoleId, UserId},
};

#[expose_input]
pub(crate) struct RawGuildUser {
    pub guild_id: String,
    pub user_id: String,
    pub reason: Option<String>,
}

#[op2(async)]
pub async fn op_kick_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGuildUser,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    http.kick_member(guild_id, user_id, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[expose_input]
pub(crate) struct RawBanMember {
    pub guild_id: String,
    pub user_id: String,
    pub delete_message_seconds: Option<u32>,
    pub reason: Option<String>,
}

#[op2(async)]
pub async fn op_ban_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawBanMember,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    let delete_seconds = args.delete_message_seconds.unwrap_or(0);
    http.ban_user(guild_id, user_id, delete_seconds, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_unban_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawGuildUser,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    http.remove_ban(guild_id, user_id, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[expose_input]
pub(crate) struct RawMemberRole {
    pub guild_id: String,
    pub user_id: String,
    pub role_id: String,
    pub reason: Option<String>,
}

#[op2(async)]
pub async fn op_add_member_role(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawMemberRole,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    let role_id = parse_role_id(&args.role_id)?;
    http.add_member_role(guild_id, user_id, role_id, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_remove_member_role(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawMemberRole,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    let role_id = parse_role_id(&args.role_id)?;
    http.remove_member_role(guild_id, user_id, role_id, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[expose_input]
pub(crate) struct RawEditMember {
    pub guild_id: String,
    pub user_id: String,
    pub payload: serde_json::Value,
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_edit_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditMember,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    let member = http
        .edit_member(guild_id, user_id, &args.payload, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(member).map_err(|err| JsErrorBox::generic(err.to_string()))
}

fn parse_guild_id(value: &str) -> Result<GuildId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(GuildId::new)
        .map_err(|_| JsErrorBox::generic("Invalid guild id"))
}

fn parse_user_id(value: &str) -> Result<UserId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(UserId::new)
        .map_err(|_| JsErrorBox::generic("Invalid user id"))
}

fn parse_role_id(value: &str) -> Result<RoleId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(RoleId::new)
        .map_err(|_| JsErrorBox::generic("Invalid role id"))
}
