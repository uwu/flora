import type {
  JsonValue,
  RawAllowedMentions,
  RawAttachment,
  RawBanMember,
  RawBulkDeleteMessages,
  RawClearReactions,
  RawCommandPermissions,
  RawCreateChannel,
  RawCreateGuildCommand,
  RawCreateThread,
  RawCreateThreadFromMessage,
  RawCrosspostMessage,
  RawDeferInteractionResponse,
  RawDeleteChannel,
  RawDeleteFollowupMessage,
  RawDeleteGuildCommand,
  RawDeleteInteractionResponse,
  RawDeleteMessage,
  RawDeleteWebhook,
  RawEditChannel,
  RawEditCurrentMember,
  RawEditGuildCommand,
  RawEditInteractionResponse,
  RawEditMember,
  RawEditMessage,
  RawEditWebhook,
  RawExecuteWebhook,
  RawFetchMessage,
  RawFetchMessages,
  RawFollowupMessage,
  RawGetGuildCommand,
  RawGuildId,
  RawGuildUser,
  RawInteractionResponse,
  RawMemberRole,
  RawPinMessage,
  RawReaction,
  RawSendMessage,
  RawThreadId,
  RawThreadMember,
  RawUpdateInteractionResponse,
  RawUpsertGuildCommands
} from '../generated'

// Lightweight REST bindings over core ops.
// These map 1:1 to Rust ops and mostly accept the Raw* payloads.

declare const Deno: {
  core: {
    ops: any
  }
}

const ops = Deno.core.ops as any

export const rest = {
  sendMessage: (args: RawSendMessage): Promise<void> => ops.op_send_message(args),
  editMessage: (args: RawEditMessage): Promise<void> => ops.op_edit_message(args),
  deleteMessage: (args: RawDeleteMessage): Promise<void> => ops.op_delete_message(args),
  bulkDeleteMessages: (args: RawBulkDeleteMessages): Promise<void> =>
    ops.op_bulk_delete_messages(args),
  pinMessage: (args: RawPinMessage): Promise<void> => ops.op_pin_message(args),
  unpinMessage: (args: RawPinMessage): Promise<void> => ops.op_unpin_message(args),
  crosspostMessage: (args: RawCrosspostMessage): Promise<JsonValue> =>
    ops.op_crosspost_message(args),
  fetchMessage: (args: RawFetchMessage): Promise<JsonValue> => ops.op_fetch_message(args),
  fetchMessages: (args: RawFetchMessages): Promise<JsonValue[]> => ops.op_fetch_messages(args),
  addReaction: (args: RawReaction): Promise<void> => ops.op_add_reaction(args),
  removeReaction: (args: RawReaction): Promise<void> => ops.op_remove_reaction(args),
  clearReactions: (args: RawClearReactions): Promise<void> => ops.op_clear_reactions(args),

  sendInteractionResponse: (args: RawInteractionResponse): Promise<void> =>
    ops.op_send_interaction_response(args),
  deferInteractionResponse: (args: RawDeferInteractionResponse): Promise<void> =>
    ops.op_defer_interaction_response(args),
  updateInteractionResponse: (args: RawUpdateInteractionResponse): Promise<void> =>
    ops.op_update_interaction_response(args),
  editOriginalInteractionResponse: (
    args: RawEditInteractionResponse
  ): Promise<JsonValue> => ops.op_edit_original_interaction_response(args),
  deleteOriginalInteractionResponse: (
    args: RawDeleteInteractionResponse
  ): Promise<void> => ops.op_delete_original_interaction_response(args),
  createFollowupMessage: (args: RawFollowupMessage): Promise<JsonValue> =>
    ops.op_create_followup_message(args),
  editFollowupMessage: (args: RawFollowupMessage): Promise<JsonValue> =>
    ops.op_edit_followup_message(args),
  deleteFollowupMessage: (args: RawDeleteFollowupMessage): Promise<void> =>
    ops.op_delete_followup_message(args),

  upsertGuildCommands: (args: RawUpsertGuildCommands): Promise<void> =>
    ops.op_upsert_guild_commands(args),
  createGuildCommand: (args: RawCreateGuildCommand): Promise<JsonValue> =>
    ops.op_create_guild_command(args),
  editGuildCommand: (args: RawEditGuildCommand): Promise<JsonValue> =>
    ops.op_edit_guild_command(args),
  deleteGuildCommand: (args: RawDeleteGuildCommand): Promise<void> =>
    ops.op_delete_guild_command(args),
  getGuildCommands: (args: RawGuildId): Promise<JsonValue[]> => ops.op_get_guild_commands(args),
  getGuildCommand: (args: RawGetGuildCommand): Promise<JsonValue> => ops.op_get_guild_command(args),
  editGuildCommandPermissions: (
    args: RawCommandPermissions
  ): Promise<JsonValue> => ops.op_edit_guild_command_permissions(args),
  getGuildCommandsPermissions: (args: RawGuildId): Promise<JsonValue[]> =>
    ops.op_get_guild_commands_permissions(args),
  getGuildCommandPermissions: (args: RawGetGuildCommand): Promise<JsonValue> =>
    ops.op_get_guild_command_permissions(args),

  kickMember: (args: RawGuildUser): Promise<void> => ops.op_kick_member(args),
  banMember: (args: RawBanMember): Promise<void> => ops.op_ban_member(args),
  unbanMember: (args: RawGuildUser): Promise<void> => ops.op_unban_member(args),
  addMemberRole: (args: RawMemberRole): Promise<void> => ops.op_add_member_role(args),
  removeMemberRole: (args: RawMemberRole): Promise<void> => ops.op_remove_member_role(args),
  editMember: (args: RawEditMember): Promise<JsonValue> => ops.op_edit_member(args),
  editCurrentMember: (args: RawEditCurrentMember): Promise<JsonValue> =>
    ops.op_edit_current_member(args),

  createChannel: (args: RawCreateChannel): Promise<JsonValue> => ops.op_create_channel(args),
  editChannel: (args: RawEditChannel): Promise<JsonValue> => ops.op_edit_channel(args),
  deleteChannel: (args: RawDeleteChannel): Promise<JsonValue> => ops.op_delete_channel(args),
  createThread: (args: RawCreateThread): Promise<JsonValue> => ops.op_create_thread(args),
  createThreadFromMessage: (args: RawCreateThreadFromMessage): Promise<JsonValue> =>
    ops.op_create_thread_from_message(args),
  joinThread: (args: RawThreadId): Promise<void> => ops.op_join_thread(args),
  leaveThread: (args: RawThreadId): Promise<void> => ops.op_leave_thread(args),
  addThreadMember: (args: RawThreadMember): Promise<void> => ops.op_add_thread_member(args),
  removeThreadMember: (args: RawThreadMember): Promise<void> => ops.op_remove_thread_member(args),

  executeWebhook: (args: RawExecuteWebhook): Promise<JsonValue | null> =>
    ops.op_execute_webhook(args),
  editWebhook: (args: RawEditWebhook): Promise<JsonValue> => ops.op_edit_webhook(args),
  deleteWebhook: (args: RawDeleteWebhook): Promise<void> => ops.op_delete_webhook(args)
}

// Re-export some shared inputs for convenience
export type { RawAllowedMentions, RawAttachment }
