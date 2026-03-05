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
  function on<E extends keyof FloraEventMap>(
    event: E,
    handler: FloraEventHandler<E>
  ): void
  function __floraDispatch(event: string, payload: unknown): Promise<void>
  function registerSlashCommands(
    commands: FlattenedSlashCommand[]
  ): Promise<void> | undefined
  function cron(
    name: string,
    cronExpr: string,
    handler: CronHandler,
    options?: CronOptions
  ): void
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
      op_register_cron(options: {
        name: string
        expr: string
        skipIfRunning?: boolean
      }): void
      op_secret_placeholder(name: string): string | undefined
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

type BlobPart = string | ArrayBuffer | ArrayBufferView

const maybeBuffer = (
  globalThis as {
    Buffer?: {
      from(
        input: string,
        encoding: string
      ): { toString(encoding: string): string }
    }
  }
).Buffer

if (typeof globalThis.btoa === 'undefined') {
  ;(globalThis as Record<string, unknown>).btoa = (input: string) =>
    maybeBuffer
      ? maybeBuffer.from(input, 'binary').toString('base64')
      : String(input)
}

if (typeof globalThis.atob === 'undefined') {
  ;(globalThis as Record<string, unknown>).atob = (input: string) =>
    maybeBuffer
      ? maybeBuffer.from(input, 'base64').toString('binary')
      : String(input)
}

if (typeof globalThis.TextEncoder === 'undefined') {
  class TextEncoderPolyfill {
    encode(input: string): Uint8Array {
      if (maybeBuffer) {
        return Uint8Array.from(
          maybeBuffer.from(input, 'utf8').toString('binary'),
          (char) => char.charCodeAt(0)
        )
      }

      const bytes = []
      for (let i = 0; i < input.length; i++) {
        bytes.push(input.charCodeAt(i) & 0xff)
      }
      return Uint8Array.from(bytes)
    }
  }

  ;(globalThis as Record<string, unknown>).TextEncoder = TextEncoderPolyfill
}

if (typeof globalThis.TextDecoder === 'undefined') {
  class TextDecoderPolyfill {
    decode(input?: ArrayBufferView | ArrayBuffer): string {
      if (!input) return ''

      const bytes = input instanceof ArrayBuffer
        ? new Uint8Array(input)
        : new Uint8Array(input.buffer, input.byteOffset, input.byteLength)

      if (maybeBuffer) {
        let binary = ''
        for (const byte of bytes) {
          binary += String.fromCharCode(byte)
        }
        return maybeBuffer.from(binary, 'binary').toString('utf8')
      }

      let text = ''
      for (const byte of bytes) {
        text += String.fromCharCode(byte)
      }
      return text
    }
  }

  ;(globalThis as Record<string, unknown>).TextDecoder = TextDecoderPolyfill
}

if (typeof globalThis.Blob === 'undefined') {
  class BlobPolyfill {
    private readonly chunks: Uint8Array[]
    readonly size: number
    readonly type: string

    constructor(parts: BlobPart[] = [], options: { type?: string } = {}) {
      this.chunks = parts.map(toUint8)
      this.size = this.chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0)
      this.type = options.type ?? ''
    }

    async arrayBuffer(): Promise<ArrayBuffer> {
      return concatChunks(this.chunks).buffer
    }

    async text(): Promise<string> {
      return new TextDecoder().decode(concatChunks(this.chunks))
    }
  }

  ;(globalThis as Record<string, unknown>).Blob = BlobPolyfill
}

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

// @ts-expect-error
globalThis.console = {
  log: (...args: unknown[]) => core.ops.op_log(args),
  warn: (...args: unknown[]) => core.ops.op_log(args)
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

function toUint8(part: BlobPart): Uint8Array {
  if (typeof part === 'string') {
    return new TextEncoder().encode(part)
  }

  if (part instanceof ArrayBuffer) {
    return new Uint8Array(part)
  }

  return new Uint8Array(part.buffer, part.byteOffset, part.byteLength)
}

function concatChunks(chunks: Uint8Array[]): Uint8Array {
  const total = chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0)
  const out = new Uint8Array(total)
  let offset = 0
  for (const chunk of chunks) {
    out.set(chunk, offset)
    offset += chunk.byteLength
  }
  return out
}
