import type { FlattenedSlashCommand } from '../sdk/commands'
import type {
  BaseContext,
  ComponentInteractionContext,
  EventReady,
  InteractionContext,
  MessageContext,
  MessageDeleteBulkContext,
  MessageDeleteContext,
  MessageEditOptions,
  MessageReplyOptions,
  MessageUpdateContext,
  ModalSubmitContext,
  ReactionContext,
  ReactionRemoveAllContext
} from '../sdk/types'
import { normalizeEdit, normalizeReply } from './normalize'
import type { AnyPayload } from './normalize'

export interface FloraEventMap {
  ready: BaseContext<EventReady>
  messageCreate: MessageContext
  messageUpdate: MessageUpdateContext
  messageDelete: MessageDeleteContext
  messageDeleteBulk: MessageDeleteBulkContext
  interactionCreate: InteractionContext
  componentInteraction: ComponentInteractionContext
  modalSubmit: ModalSubmitContext
  reactionAdd: ReactionContext
  reactionRemove: ReactionContext
  reactionRemoveEmoji: ReactionContext
  reactionRemoveAll: ReactionRemoveAllContext
}

export type FloraEventHandler<E extends keyof FloraEventMap> = (
  ctx: FloraEventMap[E]
) => void | Promise<void>

export interface CronContext {
  name: string
  scheduledAt: string
}

export interface CronOptions {
  skipIfRunning?: boolean
}

export type CronHandler = (ctx: CronContext) => void | Promise<void>

export interface Secrets {
  get(name: string): string | undefined
}

declare global {
  var __floraHandlers: Record<string, Function[]>
  var __floraGuildId: string | undefined
  function on<E extends keyof FloraEventMap>(event: E, handler: FloraEventHandler<E>): void
  function __floraDispatch(event: string, payload: unknown): Promise<void>
  function registerSlashCommands(commands: FlattenedSlashCommand[]): Promise<void> | undefined
  function cron(name: string, cronExpr: string, handler: CronHandler, options?: CronOptions): void
  var secrets: Secrets
}

declare const Deno: {
  core: {
    ops: {
      op_send_message(options: unknown): Promise<void>
      op_send_interaction_response(options: unknown): Promise<void>
      op_edit_message(options: unknown): Promise<void>
      op_log(args: unknown[]): void
      op_upsert_guild_commands(options: {
        guildId: string
        commands: FlattenedSlashCommand[]
      }): Promise<void>
      op_register_cron(options: { name: string; expr: string; skipIfRunning?: boolean }): void
      op_secret_placeholder(name: string): string | undefined
    }
  }
}

const core = Deno.core
globalThis.__floraHandlers = {}
globalThis.secrets = {
  get(name: string) {
    return core.ops.op_secret_placeholder(name)
  }
}

globalThis.on = function on<E extends keyof FloraEventMap>(
  event: E,
  handler: FloraEventHandler<E>
): void {
  if (!globalThis.__floraHandlers[event]) {
    globalThis.__floraHandlers[event] = []
  }
  globalThis.__floraHandlers[event].push(handler)
}

globalThis.__floraDispatch = async function __floraDispatch(
  event: string,
  payload: unknown
): Promise<void> {
  const handlers = globalThis.__floraHandlers[event] || []
  for (const handler of handlers) {
    const context = {
      msg: payload,
      reply(message: string | MessageReplyOptions) {
        const options = normalizeReply(message, payload as AnyPayload)
        if (options['interactionId'] && options['token']) {
          return core.ops.op_send_interaction_response(options)
        }
        return core.ops.op_send_message(options)
      },
      edit(message: string | MessageEditOptions) {
        const options = normalizeEdit(message, payload as AnyPayload)
        return core.ops.op_edit_message(options)
      }
    }
    await handler(context)
  }
}

// @ts-expect-error - Override console with minimal implementation
globalThis.console = {
  log: (...args: unknown[]) => core.ops.op_log(args)
}

globalThis.registerSlashCommands = function registerSlashCommands(
  commands: FlattenedSlashCommand[]
): Promise<void> | undefined {
  if (!globalThis.__floraGuildId) return
  return core.ops.op_upsert_guild_commands({
    guildId: globalThis.__floraGuildId,
    commands
  })
}

const CRON_EVENT_PREFIX = '__cron:'

globalThis.cron = function cron(
  name: string,
  cronExpr: string,
  handler: CronHandler,
  options?: CronOptions
): void {
  if (typeof name !== 'string' || !name.length) {
    throw new TypeError('cron name must be a non-empty string')
  }
  if (typeof cronExpr !== 'string' || !cronExpr.length) {
    throw new TypeError('cron expression must be a non-empty string')
  }
  if (typeof handler !== 'function') {
    throw new TypeError('cron handler must be a function')
  }

  const eventName = CRON_EVENT_PREFIX + name

  if (!globalThis.__floraHandlers[eventName]) {
    globalThis.__floraHandlers[eventName] = []
  }
  globalThis.__floraHandlers[eventName].push(handler)

  core.ops.op_register_cron({
    name,
    expr: cronExpr,
    skipIfRunning: options?.skipIfRunning ?? false
  })
}
