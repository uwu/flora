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

export type CronHandler = (ctx: CronContext) => void | Promise<void>

declare global {
  var __floraHandlers: Record<string, Function[]>
  var __floraGuildId: string | undefined
  function on<E extends keyof FloraEventMap>(event: E, handler: FloraEventHandler<E>): void
  function __floraDispatch(event: string, payload: unknown): Promise<void>
  function registerSlashCommands(commands: FlattenedSlashCommand[]): Promise<void> | undefined
  function cron(name: string, cronExpr: string, handler: CronHandler): void
}

declare const Deno: {
  core: {
    ops: {
      op_send_message(options: unknown): Promise<void>
      op_send_interaction_response(options: unknown): Promise<void>
      op_edit_message(options: unknown): Promise<void>
      op_log(args: unknown[]): void
      op_upsert_guild_commands(
        options: { guildId: string; commands: FlattenedSlashCommand[] }
      ): Promise<void>
      op_register_cron(options: { name: string; expr: string }): void
    }
  }
}

type AnyPayload = {
  id?: string
  messageId?: string
  channelId?: string
  interactionId?: string
  interactionToken?: string
  token?: string
  [key: string]: unknown
}

const core = Deno.core
globalThis.__floraHandlers = {}

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
  handler: CronHandler
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

  core.ops.op_register_cron({ name, expr: cronExpr })
}

function normalizeReply(
  message: string | MessageReplyOptions,
  payload: AnyPayload
): Record<string, unknown> {
  if (payload?.interactionToken) {
    return normalizeInteractionReply(message, payload)
  }

  const base = { channelId: payload.channelId }
  const replyId = payload.id ?? payload.messageId

  if (typeof message === 'string') {
    return { ...base, messageId: replyId, content: message }
  }

  if (message && typeof message === 'object') {
    const normalized: Record<string, unknown> = { ...base, ...message }
    const explicitReplyTo = message.replyTo ?? (message as Record<string, unknown>).replyTo

    if (explicitReplyTo === null) {
      delete normalized.messageId
    } else if (explicitReplyTo !== undefined) {
      normalized.messageId = explicitReplyTo
    } else if (replyId) {
      normalized.messageId = replyId
    }

    delete normalized.replyTo
    delete normalized.reply_to
    return normalized
  }

  return { ...base, messageId: replyId, content: String(message) }
}

function normalizeEdit(
  message: string | MessageEditOptions,
  payload: AnyPayload
): Record<string, unknown> {
  const messageId = payload.id ?? payload.messageId
  if (!messageId || !payload?.channelId) {
    throw new Error('Message edit requires a message payload')
  }

  const base = { channelId: payload.channelId, messageId }

  if (typeof message === 'string') {
    return { ...base, content: message }
  }

  if (message && typeof message === 'object') {
    return { ...base, ...message }
  }

  return { ...base, content: String(message) }
}

function normalizeInteractionReply(
  message: string | MessageReplyOptions,
  payload: AnyPayload
): Record<string, unknown> {
  const base = {
    interactionId: payload.interactionId ?? payload.id,
    token: payload.interactionToken
  }

  if (typeof message === 'string') {
    return { ...base, content: message }
  }

  if (message && typeof message === 'object') {
    const normalized: Record<string, unknown> = { ...base, ...message }
    if (message.ephemeral !== undefined) {
      normalized.ephemeral = message.ephemeral
    }
    return normalized
  }

  return { ...base, content: String(message) }
}
