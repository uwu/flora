// Re-export generated types from Rust
// These reflect the exact wire format used by the runtime
export type {
  // Event Payloads
  UserPayload,
  MemberPayload,
  MessagePayload,
  MessageUpdatePayload,
  MessageDeletePayload,
  MessageDeleteBulkPayload,
  InteractionCreatePayload,
  ReadyPayload,
  // Op Input Types - Interaction
  InteractionResponseArgs,
  UpsertGuildCommandsArgs,
  SlashCommandDef,
  SlashCommandOptionDef,
  // Op Input/Output Types - KV
  SetOptions,
  ListKeysOptions,
  ListKeysResult,
  KvKeyInfo,
  KvKeyMetadata,
} from './generated'

import type {
  MessagePayload,
  MessageUpdatePayload,
  MessageDeletePayload,
  MessageDeleteBulkPayload,
  InteractionCreatePayload,
  UserPayload,
  MemberPayload,
} from './generated'

// SDK-specific types with ergonomic optional fields
// These are more user-friendly than the wire format types

export type EmbedField = {
  name: string
  value: string
  inline?: boolean
}

export type Embed = {
  title?: string
  description?: string
  url?: string
  color?: number
  timestamp?: string
  footer?: { text: string; iconUrl?: string }
  image?: { url: string }
  thumbnail?: { url: string }
  author?: { name?: string; url?: string; iconUrl?: string }
  fields?: EmbedField[]
}

export type Attachment =
  | { url: string; filename?: string; description?: string }
  | { data: string; filename: string; description?: string }

export type AllowedMentions = {
  parse?: Array<'everyone' | 'roles' | 'users'>
  users?: string[]
  roles?: string[]
  repliedUser?: boolean
}

// Type aliases for convenience - match generated payload types
export type MessageAuthor = UserPayload
export type GuildMember = MemberPayload
export type InteractionPayload = InteractionCreatePayload

// EmbedBuilder class for fluent embed construction
export class EmbedBuilder {
  #embed: Embed

  constructor(initial: Embed = {}) {
    this.#embed = { ...initial }
  }

  setTitle(title: string) {
    this.#embed.title = title
    return this
  }

  setDescription(description: string) {
    this.#embed.description = description
    return this
  }

  setUrl(url: string) {
    this.#embed.url = url
    return this
  }

  setColor(color: number) {
    this.#embed.color = color
    return this
  }

  setTimestamp(timestamp: string) {
    this.#embed.timestamp = timestamp
    return this
  }

  setFooter(text: string, iconUrl?: string) {
    this.#embed.footer = { text, iconUrl }
    return this
  }

  setImage(url: string) {
    this.#embed.image = { url }
    return this
  }

  setThumbnail(url: string) {
    this.#embed.thumbnail = { url }
    return this
  }

  setAuthor(name?: string, options?: { url?: string; iconUrl?: string }) {
    this.#embed.author = { name, ...options }
    return this
  }

  addField(name: string, value: string, inline = false) {
    const field: EmbedField = { name, value, inline }
    this.#embed.fields = [...(this.#embed.fields ?? []), field]
    return this
  }

  addFields(fields: EmbedField[]) {
    this.#embed.fields = [...(this.#embed.fields ?? []), ...fields]
    return this
  }

  setFields(fields: EmbedField[]) {
    this.#embed.fields = [...fields]
    return this
  }

  toJSON(): Embed {
    return { ...this.#embed }
  }
}

export function embed(initial?: Embed) {
  return new EmbedBuilder(initial)
}

// SDK-specific types for reply/edit options
export type MessageReplyOptions = {
  content?: string
  embeds?: Embed[]
  attachments?: Attachment[]
  tts?: boolean
  allowedMentions?: AllowedMentions
  replyTo?: string | null
  ephemeral?: boolean
  flags?: number
}

export type MessageEditOptions = {
  content?: string
  embeds?: Embed[]
  allowedMentions?: AllowedMentions
  flags?: number
}

// Base context type that adds reply/edit methods to payloads
type BaseContext<TPayload> = {
  msg: TPayload
  reply: (content: string | MessageReplyOptions) => Promise<void>
  edit: (content: string | MessageEditOptions) => Promise<void>
}

// Context types for each event
export type MessageContext = BaseContext<MessagePayload>
export type MessageUpdateContext = BaseContext<MessageUpdatePayload>
export type MessageDeleteContext = BaseContext<MessageDeletePayload>
export type MessageDeleteBulkContext = BaseContext<MessageDeleteBulkPayload>

export type SlashCommandOptions = Record<string, string | number | boolean | undefined>

export type InteractionContext = BaseContext<InteractionPayload> & {
  options: SlashCommandOptions
}

// Command definition types
export type Command = {
  name: string
  description?: string
  run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
}

export function defineCommand(command: Command): Command {
  return command
}

export type SlashCommandOption = {
  name: string
  description: string
  type?: 'string' | 'integer' | 'number' | 'boolean' | 'subcommand' | 'subcommand_group'
  required?: boolean
  options?: SlashCommandOption[]
}

export type SlashSubcommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
  run: (ctx: InteractionContext) => Promise<void> | void
}

export type SlashCommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
  subcommands?: SlashSubcommand[]
  run?: (ctx: InteractionContext) => Promise<void> | void
}

export function defineSlashCommand(command: SlashCommand): SlashCommand {
  return command
}

type CreateOptions = {
  prefix?: string
  commands?: Command[]
  prefixCommands?: Command[]
  slashCommands?: SlashCommand[]
}

export function createBot(options: CreateOptions) {
  const prefix = options.prefix ?? '!'
  const commands = options.commands ?? options.prefixCommands ?? []
  const slashCommands = options.slashCommands ?? []

  on('messageCreate', async (ctx: MessageContext) => {
    if (!ctx.msg || !ctx.msg.content) return
    if (ctx.msg.author?.bot) return

    const content = ctx.msg.content.trim()
    if (!content.startsWith(prefix)) return

    const body = content.slice(prefix.length).trim()
    const [commandName, ...args] = body.split(/\s+/)
    const command = commands.find((cmd) => cmd.name === commandName)
    if (!command) return

    await command.run({ ...ctx, args })
  })

  on('interactionCreate', async (ctx: InteractionContext) => {
    if (!ctx.msg) return
    const command = slashCommands.find((cmd) => cmd.name === ctx.msg.command_name)
    if (!command) return

    if (command.subcommands && command.subcommands.length > 0) {
      await handleSubcommand(ctx, command)
    } else if (command.run) {
      // Flatten options for non-subcommand slash commands
      const rawData = ctx.msg.data as any
      const options = flattenInteractionOptions(rawData?.options || [])
      await command.run({ ...ctx, options })
    }
  })

  if (slashCommands.length && typeof registerSlashCommands === 'function') {
    const flattenedCommands = flattenCommands(slashCommands)
    registerSlashCommands(flattenedCommands)
  }
}

type FlattenedSlashCommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
}

type SubcommandMap = Record<string, Record<string, (ctx: InteractionContext) => Promise<void> | void>>

// Note: kv and EmbedBuilder are exported from this module and also exposed as globals
// via the IIFE footer. The global type declarations for them are in the generated
// flora-globals.d.ts file to avoid redeclaration conflicts.

declare global {
  // Internal runtime state
  var __floraSubcommands: SubcommandMap

  // Event handlers (typed overloads)
  function on(event: 'messageCreate', handler: (ctx: MessageContext) => void | Promise<void>): void
  function on(event: 'messageUpdate', handler: (ctx: MessageUpdateContext) => void | Promise<void>): void
  function on(event: 'messageDelete', handler: (ctx: MessageDeleteContext) => void | Promise<void>): void
  function on(event: 'messageDeleteBulk', handler: (ctx: MessageDeleteBulkContext) => void | Promise<void>): void
  function on(event: 'interactionCreate', handler: (ctx: InteractionContext) => void | Promise<void>): void
  function on(event: string, handler: (ctx: any) => void | Promise<void>): void
  function registerSlashCommands(commands: FlattenedSlashCommand[]): void

  // SDK functions (exposed via IIFE footer)
  function createBot(options: CreateOptions): void
  function defineCommand(command: Command): Command
  function defineSlashCommand(command: SlashCommand): SlashCommand
  function hasRole(ctx: InteractionContext, roleId: string): boolean
  function getSubcommand(ctx: InteractionContext): string | undefined
  function getSubcommandGroup(ctx: InteractionContext): string | undefined
  function embed(initial?: Embed): EmbedBuilder
}

function flattenCommands(commands: SlashCommand[]): FlattenedSlashCommand[] {
  globalThis.__floraSubcommands = globalThis.__floraSubcommands || {}
  return commands.map((cmd) => {
    if (cmd.subcommands && cmd.subcommands.length > 0) {
      const submap: Record<string, (ctx: InteractionContext) => Promise<void> | void> = {}
      cmd.subcommands.forEach((sub) => {
        submap[sub.name] = sub.run
      })
      globalThis.__floraSubcommands[cmd.name] = submap

      return {
        name: cmd.name,
        description: cmd.description,
        options: cmd.subcommands.map((sub) => ({
          name: sub.name,
          description: sub.description,
          type: 'subcommand' as const,
          options: sub.options
        }))
      }
    }

    return {
      name: cmd.name,
      description: cmd.description,
      options: cmd.options
    }
  })
}

async function handleSubcommand(ctx: InteractionContext, command: SlashCommand) {
  const rawData = ctx.msg.data as any
  if (!rawData?.options || !Array.isArray(rawData.options)) {
    return
  }

  const firstOption = rawData.options[0]
  if (!firstOption) return

  const subcommandName = firstOption.name
  const subcommandMap = globalThis.__floraSubcommands?.[command.name]
  if (!subcommandMap) return

  const subcommandHandler = subcommandMap[subcommandName]
  if (!subcommandHandler) return

  const subcommandOptions = firstOption.options || []
  const flatOptions = flattenInteractionOptions(subcommandOptions)

  const enrichedCtx = {
    ...ctx,
    options: flatOptions
  }

  await subcommandHandler(enrichedCtx)
}

function flattenInteractionOptions(options: any[]): Record<string, any> {
  const result: Record<string, any> = {}

  for (const opt of options) {
    if (opt.type === 1 || opt.type === 2) {
      Object.assign(result, flattenInteractionOptions(opt.options || []))
    } else {
      result[opt.name] = opt.value
    }
  }

  return result
}

export function hasRole(ctx: InteractionContext, roleId: string): boolean {
  return ctx.msg.member?.roles?.includes(roleId) ?? false
}

export function getSubcommand(ctx: InteractionContext): string | undefined {
  const rawData = ctx.msg.data as any
  if (!rawData?.options || !Array.isArray(rawData.options)) return undefined
  return rawData.options[0]?.name
}

export function getSubcommandGroup(ctx: InteractionContext): string | undefined {
  const rawData = ctx.msg.data as any
  if (!rawData?.options || !Array.isArray(rawData.options)) return undefined

  const firstOption = rawData.options[0]
  if (!firstOption) return undefined

  const type = firstOption.type
  if (type === 2) {
    return firstOption.name
  }

  return undefined
}

// Export KV API
export { kv, KvStore } from './kv'
