// Re-export all types from generated
import type {
  EventComponentInteraction,
  EventInteractionCreate,
  EventMessage,
  EventMessageDelete,
  EventMessageDeleteBulk,
  EventMessageUpdate,
  EventModalSubmit,
  EventReaction,
  EventReactionRemoveAll,
  JsonValue,
  RawAllowedMentions,
  RawAttachment,
  RawEmbed,
  RawEmbedField
} from '../generated'
export * from '../generated'

export type Embed = RawEmbed
export type EmbedField = RawEmbedField

export type MessageReplyOptions = {
  content?: string
  embeds?: RawEmbed[]
  attachments?: RawAttachment[]
  components?: JsonValue[]
  tts?: boolean
  allowedMentions?: RawAllowedMentions
  replyTo?: string | null
  ephemeral?: boolean
  flags?: number
}

export type MessageEditOptions = {
  content?: string
  embeds?: RawEmbed[]
  components?: JsonValue[]
  allowedMentions?: RawAllowedMentions
  flags?: number
}

export type BaseContext<TPayload> = {
  msg: TPayload
  reply: (content: string | MessageReplyOptions) => Promise<void>
  edit: (content: string | MessageEditOptions) => Promise<void>
}

export type MessageContext = BaseContext<EventMessage>
export type MessageUpdateContext = BaseContext<EventMessageUpdate>
export type MessageDeleteContext = BaseContext<EventMessageDelete>
export type MessageDeleteBulkContext = BaseContext<EventMessageDeleteBulk>
export type ComponentInteractionContext = BaseContext<EventComponentInteraction>
export type ModalSubmitContext = BaseContext<EventModalSubmit>
export type ReactionContext = BaseContext<EventReaction>
export type ReactionRemoveAllContext = BaseContext<EventReactionRemoveAll>

export type SlashCommandOptions = Record<string, string | number | boolean | undefined>

export type InteractionContext = BaseContext<EventInteractionCreate> & {
  options: SlashCommandOptions
}
