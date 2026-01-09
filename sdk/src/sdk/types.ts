// Re-export all types from generated
import type { EventInteractionCreate, EventMessage, EventMessageDelete, EventMessageDeleteBulk, EventMessageUpdate, RawAllowedMentions, RawAttachment, RawEmbed } from '../generated'
export type * from '../generated'

export type MessageReplyOptions = {
  content?: string
  embeds?: RawEmbed[]
  attachments?: RawAttachment[]
  tts?: boolean
  allowedMentions?: RawAllowedMentions
  replyTo?: string | null
  ephemeral?: boolean
  flags?: number
}

export type MessageEditOptions = {
  content?: string
  embeds?: RawEmbed[]
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

export type SlashCommandOptions = Record<
  string,
  string | number | boolean | undefined
>

export type InteractionContext = BaseContext<EventInteractionCreate> & {
  options: SlashCommandOptions
}
