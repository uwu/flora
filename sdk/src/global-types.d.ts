// Global type declarations for Flora SDK
// These types are available globally in user scripts without imports

import type {
  MessagePayload,
  MessageUpdatePayload,
  MessageDeletePayload,
  MessageDeleteBulkPayload,
  InteractionCreatePayload,
  UserPayload,
  MemberPayload,
  ListKeysResult,
  KvKeyInfo,
} from './generated'

import type {
  Embed,
  EmbedField,
  Attachment,
  AllowedMentions,
  MessageReplyOptions,
  MessageEditOptions,
  Command,
  SlashCommand,
  SlashCommandOption,
  SlashSubcommand,
  EmbedBuilder as EmbedBuilderClass,
} from './index'

// Base context type
type BaseContext<TPayload> = {
  msg: TPayload
  reply: (content: string | MessageReplyOptions) => Promise<void>
  edit: (content: string | MessageEditOptions) => Promise<void>
}

// Context types
type MessageContext = BaseContext<MessagePayload>
type MessageUpdateContext = BaseContext<MessageUpdatePayload>
type MessageDeleteContext = BaseContext<MessageDeletePayload>
type MessageDeleteBulkContext = BaseContext<MessageDeleteBulkPayload>

type SlashCommandOptions = Record<string, string | number | boolean | undefined>
type InteractionContext = BaseContext<InteractionCreatePayload> & {
  options: SlashCommandOptions
}

// KV Store types
interface KvStore {
  get(key: string): Promise<string | null>
  getWithMetadata(key: string): Promise<{ value: string | null; metadata?: Record<string, unknown> }>
  set(key: string, value: string, options?: {
    expiration?: number
    metadata?: Record<string, unknown>
  }): Promise<void>
  updateMetadata(key: string, metadata: Record<string, unknown> | null): Promise<void>
  delete(key: string): Promise<void>
  list(options?: {
    prefix?: string
    limit?: number
    cursor?: string
  }): Promise<ListKeysResult>
}

declare global {
  // Type aliases available globally
  type MessageAuthor = UserPayload
  type GuildMember = MemberPayload
  type InteractionPayload = InteractionCreatePayload

  // SDK types
  type EmbedBuilder = EmbedBuilderClass

  // Event handlers
  function on(
    event: 'messageCreate',
    handler: (ctx: MessageContext) => void | Promise<void>
  ): void
  function on(
    event: 'messageUpdate',
    handler: (ctx: MessageUpdateContext) => void | Promise<void>
  ): void
  function on(
    event: 'messageDelete',
    handler: (ctx: MessageDeleteContext) => void | Promise<void>
  ): void
  function on(
    event: 'messageDeleteBulk',
    handler: (ctx: MessageDeleteBulkContext) => void | Promise<void>
  ): void
  function on(
    event: 'interactionCreate',
    handler: (ctx: InteractionContext) => void | Promise<void>
  ): void

  // Bot creation
  function createBot(options: {
    prefix?: string
    commands?: Command[]
    prefixCommands?: Command[]
    slashCommands?: SlashCommand[]
  }): void

  // Command definition helpers
  function defineCommand(command: {
    name: string
    description?: string
    run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
  }): Command

  function defineSlashCommand(command: {
    name: string
    description: string
    options?: SlashCommandOption[]
    subcommands?: SlashSubcommand[]
    run?: (ctx: InteractionContext) => Promise<void> | void
  }): SlashCommand

  // Utility functions
  function hasRole(ctx: InteractionContext, roleId: string): boolean
  function getSubcommand(ctx: InteractionContext): string | undefined
  function getSubcommandGroup(ctx: InteractionContext): string | undefined

  // Embed builder
  const EmbedBuilder: new (initial?: Embed) => EmbedBuilderClass
  function embed(initial?: Embed): EmbedBuilderClass

  // KV store
  const kv: {
    store(name: string): KvStore
  }
}

export {}
