use std::sync::Arc;

use serenity::http::Http;

use crate::kv::KvService;

pub mod interaction;
pub mod kv;
pub mod message;
mod tls;
mod components;
pub mod commands;
pub mod guilds;
pub mod channels;
pub mod webhooks;

deno_core::extension!(
    flora_ops,
    ops = [
        message::op_log,
        message::op_send_message,
        message::op_edit_message,
        message::op_delete_message,
        message::op_bulk_delete_messages,
        message::op_pin_message,
        message::op_unpin_message,
        message::op_crosspost_message,
        message::op_fetch_message,
        message::op_fetch_messages,
        message::op_add_reaction,
        message::op_remove_reaction,
        message::op_clear_reactions,
        interaction::op_send_interaction_response,
        interaction::op_defer_interaction_response,
        interaction::op_update_interaction_response,
        interaction::op_edit_original_interaction_response,
        interaction::op_delete_original_interaction_response,
        interaction::op_create_followup_message,
        interaction::op_edit_followup_message,
        interaction::op_delete_followup_message,
        interaction::op_upsert_guild_commands,
        commands::op_create_guild_command,
        commands::op_edit_guild_command,
        commands::op_delete_guild_command,
        commands::op_get_guild_commands,
        commands::op_get_guild_command,
        commands::op_edit_guild_command_permissions,
        commands::op_get_guild_commands_permissions,
        commands::op_get_guild_command_permissions,
        guilds::op_kick_member,
        guilds::op_ban_member,
        guilds::op_unban_member,
        guilds::op_add_member_role,
        guilds::op_remove_member_role,
        guilds::op_edit_member,
        channels::op_create_channel,
        channels::op_edit_channel,
        channels::op_delete_channel,
        channels::op_create_thread,
        channels::op_create_thread_from_message,
        channels::op_join_thread,
        channels::op_leave_thread,
        channels::op_add_thread_member,
        channels::op_remove_thread_member,
        webhooks::op_execute_webhook,
        webhooks::op_edit_webhook,
        webhooks::op_delete_webhook,
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
