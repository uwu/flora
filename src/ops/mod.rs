use std::sync::Arc;

use serenity::http::Http;

mod interaction;
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
    ],
    options = { http: Arc<Http> },
    state = |state, options| {
        state.put(options.http.clone());
    }
);

pub fn extension(http: Arc<Http>) -> deno_core::Extension {
    flora_ops::init(http)
}
