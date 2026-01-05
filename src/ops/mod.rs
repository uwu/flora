use std::sync::Arc;

use serenity::http::Http;

use crate::kv::KvService;

mod interaction;
mod kv;
mod message;
mod tls;

deno_core::extension!(
    flora_ops,
    ops = [
        message::op_log,
        message::op_send_message,
        message::op_edit_message,
        interaction::op_send_interaction_response,
        interaction::op_upsert_guild_commands,
        tls::op_tls_peer_certificate,
        kv::op_kv_get,
        kv::op_kv_set,
        kv::op_kv_delete,
        kv::op_kv_list_keys,
        kv::op_kv_get_with_metadata,
        kv::op_kv_update_metadata,
    ],
    options = {
        http: Arc<Http>,
        kv: KvService,
    },
    state = |state, options| {
        state.put(options.http.clone());
        state.put(options.kv.clone());
    }
);

pub fn extension(http: Arc<Http>, kv: KvService) -> deno_core::Extension {
    flora_ops::init(http, kv)
}
