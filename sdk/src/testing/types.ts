import type {
  EventComponentInteraction,
  EventInteractionCreate,
  EventMessage,
  EventModalSubmit,
  EventReaction,
  EventReady
} from '../generated'

export type OpCall = { op: string; args: unknown[]; timestamp: number }

export type DispatchResult = {
  calls: OpCall[]
  replies: OpCall[]
  edits: OpCall[]
  defers: OpCall[]
  followups: OpCall[]
  logs: unknown[][]
  firstReply: unknown | undefined
}

export type InteractionOptions = Record<string, string | number | boolean>

export type HarnessOptions = {
  guildId?: string
  secrets?: Record<string, string>
}

export type DeepPartial<T> = T extends object ? { [K in keyof T]?: DeepPartial<T[K]> }
  : T

export type MessagePartial = DeepPartial<EventMessage>
export type InteractionPartial = DeepPartial<EventInteractionCreate>
export type ComponentInteractionPartial = DeepPartial<EventComponentInteraction>
export type ModalSubmitPartial = DeepPartial<EventModalSubmit>
export type ReactionPartial = DeepPartial<EventReaction>
export type ReadyPartial = DeepPartial<EventReady>
