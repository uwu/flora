import type { Command, SlashCommand, SlashCommandOption } from './commands'
import type {
  InteractionContext,
  MessageContext,
  MessageDeleteBulkContext,
  MessageDeleteContext,
  MessageEditOptions,
  MessageReplyOptions,
  MessageUpdateContext
} from './types'

// Note: kv and EmbedBuilder are exported from this module and also exposed as globals
// via the IIFE footer. The global type declarations for them are in the generated
// flora-globals.d.ts file to avoid redeclaration conflicts.

type CreateOptions = {
  prefix?: string
  commands?: Command[]
  prefixCommands?: Command[]
  slashCommands?: SlashCommand[]
}

type FlattenedSlashCommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
}

type SubcommandMap = Record<
  string,
  Record<string, (ctx: InteractionContext) => Promise<void> | void>
>

declare global {
  const __floraSubcommandMap: SubcommandMap

  // Event handlers (typed overloads)
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
  function on(event: string, handler: (ctx: any) => void | Promise<void>): void
  function registerSlashCommands(commands: FlattenedSlashCommand[]): void

  // SDK functions (exposed via IIFE footer)
  function createBot(options: CreateOptions): void
  function prefix(command: Command): Command
  function slash(command: SlashCommand): SlashCommand
  function hasRole(ctx: InteractionContext, roleId: string): boolean
  function getSubcommand(ctx: InteractionContext): string | undefined
  function getSubcommandGroup(ctx: InteractionContext): string | undefined
  function embed(initial?: any): any
}

export {}
