// Import globals to register global types
import './globals'

// Re-export types
export type {
  AllowedMentionsInput,
  AttachmentInput,
  EditMessageArgs,
  EmbedAuthorInput,
  EmbedFieldInput,
  EmbedFooterInput,
  EmbedInput,
  EmbedMediaInput,
  GuildMember,
  InteractionContext,
  InteractionCreatePayload,
  InteractionPayload,
  InteractionResponseArgs,
  KvKeyInfo,
  KvKeyMetadata,
  ListKeysOptions,
  ListKeysResult,
  MemberPayload,
  MessageAuthor,
  MessageContext,
  MessageDeleteBulkContext,
  MessageDeleteBulkPayload,
  MessageDeleteContext,
  MessageDeletePayload,
  MessageEditOptions,
  MessagePayload,
  MessageReplyOptions,
  MessageUpdateContext,
  MessageUpdatePayload,
  ReadyPayload,
  SendMessageArgs,
  SetOptions,
  SlashCommandDef,
  SlashCommandOptionDef,
  SlashCommandOptions,
  UpsertGuildCommandsArgs,
  UserPayload
} from './types'

export type * from './commands'
export type * from './embed'

// Re-export classes and functions
export { createBot, defineCommand, defineSlashCommand } from './commands'
export { embed, EmbedBuilder } from './embed'
export { getSubcommand, getSubcommandGroup, hasRole } from './helpers'

// Re-export KV API
export { kv, KvStore } from './kv'
