// Re-export all types from generated
import type {
  AllowedMentionsInput,
  AttachmentInput,
  EditMessageArgs,
  EmbedAuthorInput,
  EmbedFieldInput,
  EmbedFooterInput,
  EmbedInput,
  EmbedMediaInput,
  InteractionCreatePayload,
  // Op Input Types - Interaction
  InteractionResponseArgs,
  KvKeyInfo,
  KvKeyMetadata,
  ListKeysOptions,
  ListKeysResult,
  MemberPayload,
  MessageDeleteBulkPayload,
  MessageDeletePayload,
  MessagePayload,
  MessageUpdatePayload,
  ReadyPayload,
  // Op Input Types - Message
  SendMessageArgs,
  // Op Input/Output Types - KV
  SetOptions,
  SlashCommandDef,
  SlashCommandOptionDef,
  UpsertGuildCommandsArgs,
  // Event Payloads
  UserPayload
} from '../generated'

export type {
  AllowedMentionsInput,
  AttachmentInput,
  EditMessageArgs,
  EmbedAuthorInput,
  EmbedFieldInput,
  EmbedFooterInput,
  EmbedInput,
  EmbedMediaInput,
  InteractionCreatePayload,
  // Op Input Types - Interaction
  InteractionResponseArgs,
  KvKeyInfo,
  KvKeyMetadata,
  ListKeysOptions,
  ListKeysResult,
  MemberPayload,
  MessageDeleteBulkPayload,
  MessageDeletePayload,
  MessagePayload,
  MessageUpdatePayload,
  ReadyPayload,
  // Op Input Types - Message
  SendMessageArgs,
  // Op Input/Output Types - KV
  SetOptions,
  SlashCommandDef,
  SlashCommandOptionDef,
  UpsertGuildCommandsArgs,
  // Event Payloads
  UserPayload
} from '../generated'

export type MessageAuthor = UserPayload
export type GuildMember = MemberPayload
export type InteractionPayload = InteractionCreatePayload

export type MessageReplyOptions = {
  content?: string
  embeds?: EmbedInput[]
  attachments?: AttachmentInput[]
  tts?: boolean
  allowedMentions?: AllowedMentionsInput
  replyTo?: string | null
  ephemeral?: boolean
  flags?: number
}

export type MessageEditOptions = {
  content?: string
  embeds?: EmbedInput[]
  allowedMentions?: AllowedMentionsInput
  flags?: number
}

type BaseContext<TPayload> = {
  msg: TPayload
  reply: (content: string | MessageReplyOptions) => Promise<void>
  edit: (content: string | MessageEditOptions) => Promise<void>
}

export type MessageContext = BaseContext<MessagePayload>
export type MessageUpdateContext = BaseContext<MessageUpdatePayload>
export type MessageDeleteContext = BaseContext<MessageDeletePayload>
export type MessageDeleteBulkContext = BaseContext<MessageDeleteBulkPayload>

export type SlashCommandOptions = Record<
  string,
  string | number | boolean | undefined
>

export type InteractionContext = BaseContext<InteractionPayload> & {
  options: SlashCommandOptions
}
