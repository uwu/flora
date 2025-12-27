use std::{cell::RefCell, rc::Rc};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;

use crate::kv::KvService;

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

    kv.get(&guild_id, &store_name, &key).await.map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[op2(async)]
pub async fn op_kv_set(
    state: Rc<RefCell<OpState>>,
    #[string] store_name: String,
    #[string] key: String,
    #[string] value: String,
) -> Result<(), JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    kv.set(&guild_id, &store_name, &key, &value)
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

    kv.delete(&guild_id, &store_name, &key).await.map_err(|e| JsErrorBox::generic(e.to_string()))
}

#[op2(async)]
#[serde]
pub async fn op_kv_list_keys(
    state: Rc<RefCell<OpState>>,
    #[string] store_name: String,
    #[string] prefix: Option<String>,
) -> Result<Vec<String>, JsErrorBox> {
    let (kv, guild_id) = {
        let state = state.borrow();
        let kv = state.borrow::<KvService>().clone();
        let guild_id = get_guild_id(&state)?;
        (kv, guild_id)
    };

    kv.list_keys(&guild_id, &store_name, prefix.as_deref())
        .await
        .map_err(|e| JsErrorBox::generic(e.to_string()))
}

fn get_guild_id(state: &OpState) -> Result<String, JsErrorBox> {
    // Guild ID is stored in OpState for guild-scoped runtimes
    state
        .try_borrow::<String>()
        .ok_or_else(|| JsErrorBox::generic("guild context not available"))
        .map(|s| s.clone())
}
