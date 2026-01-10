use std::{cell::RefCell, rc::Rc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use flora_macros::expose_input;

use crate::kv::{KvService, RawKvKeyMetadata};

#[op2(async)]
#[string]
pub async fn op_kv_get(
    state: Rc<RefCell<OpState>>,
    #[string] store_name: String,
    #[string] key: String,
) -> Result<Option<String>, JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    kv.get(&guild_id, &store_name, &key)
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[expose_input]
pub struct RawKvSetOptions {
    expiration: Option<i64>,
    metadata: Option<serde_json::Value>,
}

#[op2(async)]
pub async fn op_kv_set(
    state: Rc<RefCell<OpState>>,
    #[string] store_name: String,
    #[string] key: String,
    #[string] value: String,
    #[serde(default)] options: Option<RawKvSetOptions>,
) -> Result<(), JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    let expiration = options.as_ref().and_then(|o| o.expiration);
    let metadata = options.as_ref().and_then(|o| o.metadata.clone());

    kv.set(&guild_id, &store_name, &key, &value, expiration, metadata)
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[op2(async)]
pub async fn op_kv_delete(
    state: Rc<RefCell<OpState>>,
    #[string] store_name: String,
    #[string] key: String,
) -> Result<(), JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    kv.delete(&guild_id, &store_name, &key)
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[expose_input]
pub struct RawKvListKeysOptions {
    prefix: Option<String>,
    limit: Option<i64>,
    cursor: Option<String>,
}

#[op2(async)]
#[serde]
pub async fn op_kv_list_keys(
    state: Rc<RefCell<OpState>>,
    #[serde(default)] options: Option<RawKvListKeysOptions>,
    #[string] store_name: String,
) -> Result<crate::kv::RawKvListKeysResult, JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    let prefix = options.as_ref().and_then(|o| o.prefix.as_deref());
    let limit = options.as_ref().and_then(|o| o.limit.map(|l| l as u32));
    let cursor = options.as_ref().and_then(|o| o.cursor.as_deref());

    kv.list_keys(&guild_id, &store_name, prefix, limit, cursor)
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[op2(async)]
#[serde]
pub async fn op_kv_get_with_metadata(
    state: Rc<RefCell<OpState>>,
    #[string] store_name: String,
    #[string] key: String,
) -> Result<Option<(String, Option<RawKvKeyMetadata>)>, JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    kv.get_with_metadata(&guild_id, &store_name, &key)
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[op2(async)]
pub async fn op_kv_update_metadata(
    state: Rc<RefCell<OpState>>,
    #[string] store_name: String,
    #[string] key: String,
    #[serde(default)] metadata: Option<serde_json::Value>,
) -> Result<(), JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    kv.update_metadata(&guild_id, &store_name, &key, metadata)
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))
}

fn get_guild_id(state: &OpState) -> Result<String, JsErrorBox> {
    state
        .try_borrow::<String>()
        .ok_or_else(|| JsErrorBox::generic("guild context not available"))
        .map(|s| s.clone())
}
