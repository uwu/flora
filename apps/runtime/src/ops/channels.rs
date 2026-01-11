use std::{cell::RefCell, rc::Rc, sync::Arc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;
use serenity::{
    http::Http,
    model::id::{ChannelId, GuildId, MessageId, ThreadId, UserId},
};

#[expose_input]
pub(crate) struct RawCreateChannel {
    pub guild_id: String,
    pub payload: serde_json::Value,
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_create_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateChannel,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let guild_id = parse_guild_id(&args.guild_id)?;
    let channel = http
        .create_channel(guild_id, &args.payload, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(channel).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub(crate) struct RawEditChannel {
    pub channel_id: String,
    pub payload: serde_json::Value,
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_edit_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawEditChannel,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let channel = http
        .edit_channel(channel_id.widen(), &args.payload, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(channel).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub(crate) struct RawDeleteChannel {
    pub channel_id: String,
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_delete_channel(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawDeleteChannel,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let channel = http
        .delete_channel(channel_id.widen(), args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(channel).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub(crate) struct RawCreateThread {
    pub channel_id: String,
    pub payload: serde_json::Value,
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_create_thread(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateThread,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let thread = http
        .create_thread(channel_id, &args.payload, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(thread).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub(crate) struct RawCreateThreadFromMessage {
    pub channel_id: String,
    pub message_id: String,
    pub payload: serde_json::Value,
    pub reason: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_create_thread_from_message(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawCreateThreadFromMessage,
) -> Result<serde_json::Value, JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let channel_id = parse_channel_id(&args.channel_id)?;
    let message_id = parse_message_id(&args.message_id)?;
    let thread = http
        .create_thread_from_message(channel_id, message_id, &args.payload, args.reason.as_deref())
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    serde_json::to_value(thread).map_err(|err| JsErrorBox::generic(err.to_string()))
}

#[expose_input]
pub(crate) struct RawThreadId {
    pub thread_id: String,
}

#[op2(async)]
pub async fn op_join_thread(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadId,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    http.join_thread_channel(thread_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_leave_thread(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadId,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    http.leave_thread_channel(thread_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[expose_input]
pub(crate) struct RawThreadMember {
    pub thread_id: String,
    pub user_id: String,
}

#[op2(async)]
pub async fn op_add_thread_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadMember,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    http.add_thread_channel_member(thread_id, user_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

#[op2(async)]
pub async fn op_remove_thread_member(
    state: Rc<RefCell<OpState>>,
    #[serde] args: RawThreadMember,
) -> Result<(), JsErrorBox> {
    let http = {
        let state = state.borrow();
        state.borrow::<Arc<Http>>().clone()
    };
    let thread_id = parse_thread_id(&args.thread_id)?;
    let user_id = parse_user_id(&args.user_id)?;
    http.remove_thread_channel_member(thread_id, user_id)
        .await
        .map_err(|err| JsErrorBox::generic(err.to_string()))?;
    Ok(())
}

fn parse_guild_id(value: &str) -> Result<GuildId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(GuildId::new)
        .map_err(|_| JsErrorBox::generic("Invalid guild id"))
}

fn parse_channel_id(value: &str) -> Result<ChannelId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(ChannelId::new)
        .map_err(|_| JsErrorBox::generic("Invalid channel id"))
}

fn parse_message_id(value: &str) -> Result<MessageId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(MessageId::new)
        .map_err(|_| JsErrorBox::generic("Invalid message id"))
}

fn parse_thread_id(value: &str) -> Result<ThreadId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(ThreadId::new)
        .map_err(|_| JsErrorBox::generic("Invalid thread id"))
}

fn parse_user_id(value: &str) -> Result<UserId, JsErrorBox> {
    value
        .parse::<u64>()
        .map(UserId::new)
        .map_err(|_| JsErrorBox::generic("Invalid user id"))
}
