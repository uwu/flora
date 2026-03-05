import { EventEmitter } from 'node:events'

const TOKEN_MARKER = '__FLORA_THIRDPARTY_DISCORD_TOKEN__'
const GATEWAY_URL = 'wss://gateway.discord.gg'

type RestRequestOptions = {
  fullRoute: string
  method: string
  body?: unknown
  query?: URLSearchParams | Record<string, unknown>
}

type RestRouteBody = {
  content?: string
  embeds?: unknown[]
  components?: unknown[]
  allowed_mentions?: unknown
  flags?: number
  tts?: boolean
  message_reference?: {
    message_id?: string
  }
}

export const DefaultUserAgentAppendix = 'flora-runtime'

export const DefaultRestOptions = {
  api: 'https://discord.com/api',
  authPrefix: 'Bot',
  headers: {},
  invalidRequestWarningInterval: 0,
  globalRequestsPerSecond: 50,
  offset: 50,
  rejectOnRateLimit: null,
  retries: 0,
  retryBackoff: 0,
  timeout: 15_000,
  userAgentAppendix: DefaultUserAgentAppendix,
  version: 10,
  hashSweepInterval: 0,
  hashLifetime: 0,
  handlerSweepInterval: 0
} as const

export enum RESTEvents {
  Debug = 'restDebug',
  HandlerSweep = 'handlerSweep',
  HashSweep = 'hashSweep',
  InvalidRequestWarning = 'invalidRequestWarning',
  RateLimited = 'rateLimited',
  Response = 'response'
}

export function makeURLSearchParams(
  options?: Readonly<Record<string, unknown>>
): URLSearchParams {
  const params = new URLSearchParams()
  if (!options) return params

  for (const [key, value] of Object.entries(options)) {
    const serialized = serializeSearchParam(value)
    if (serialized !== null) params.append(key, serialized)
  }

  return params
}

export function calculateUserDefaultAvatarIndex(userId: string): number {
  return Number(BigInt(userId) >> 22n) % 6
}

export class REST extends EventEmitter {
  options: Record<string, unknown>
  #token: string | null = null
  #syntheticId = 0

  constructor(options: Record<string, unknown> = {}) {
    super()
    this.options = {
      ...DefaultRestOptions,
      ...options
    }
  }

  setToken(token: string | null) {
    this.#token = typeof token === 'string'
      ? token.replace(/^bot\s*/i, '')
      : token
    return this
  }

  async get(fullRoute: string, options: Record<string, unknown> = {}) {
    return this.request({ ...options, fullRoute, method: 'GET' })
  }

  async post(fullRoute: string, options: Record<string, unknown> = {}) {
    return this.request({ ...options, fullRoute, method: 'POST' })
  }

  async put(fullRoute: string, options: Record<string, unknown> = {}) {
    return this.request({ ...options, fullRoute, method: 'PUT' })
  }

  async patch(fullRoute: string, options: Record<string, unknown> = {}) {
    return this.request({ ...options, fullRoute, method: 'PATCH' })
  }

  async delete(fullRoute: string, options: Record<string, unknown> = {}) {
    return this.request({ ...options, fullRoute, method: 'DELETE' })
  }

  async queueRequest(options: RestRequestOptions) {
    return this.request(options)
  }

  async request(options: RestRequestOptions): Promise<unknown> {
    const route = normalizeRoute(options.fullRoute)

    if (options.method === 'GET' && route === '/gateway/bot') {
      this.assertMarkerToken()
      return {
        url: GATEWAY_URL,
        shards: 1,
        session_start_limit: {
          total: 1000,
          remaining: 1000,
          reset_after: 60_000,
          max_concurrency: 1
        }
      }
    }

    const createMatch = /^\/channels\/(\d+)\/messages\/?$/.exec(route)
    if (options.method === 'POST' && createMatch) {
      const [, channelId] = createMatch
      if (!channelId) {
        throw new Error(`invalid channel route: ${route}`)
      }
      this.assertMarkerToken()
      return this.sendChannelMessage(channelId, options.body)
    }

    const editMatch = /^\/channels\/(\d+)\/messages\/(\d+)\/?$/.exec(route)
    if (options.method === 'PATCH' && editMatch) {
      const [, channelId, messageId] = editMatch
      if (!channelId || !messageId) {
        throw new Error(`invalid message route: ${route}`)
      }
      this.assertMarkerToken()
      return this.editChannelMessage(channelId, messageId, options.body)
    }

    this.emit(RESTEvents.Debug, `[flora-rest] unhandled ${options.method} ${route}`)
    return {}
  }

  private assertMarkerToken() {
    if (this.#token === TOKEN_MARKER) {
      return
    }

    const err = new Error('An invalid token was provided.')
    ;(err as { code?: string }).code = 'TokenInvalid'
    throw err
  }

  private async sendChannelMessage(channelId: string, body: unknown) {
    const payload = unwrapBody(body)
    const ops = getCoreOps()

    const sendPayload: Record<string, unknown> = { channelId }
    const content = typeof payload.content === 'string'
      ? payload.content
      : undefined
    if (content !== undefined) sendPayload.content = content
    if (Array.isArray(payload.embeds)) sendPayload.embeds = payload.embeds
    if (Array.isArray(payload.components)) sendPayload.components = payload.components
    if (payload.allowed_mentions !== undefined) {
      sendPayload.allowedMentions = payload.allowed_mentions
    }
    if (typeof payload.flags === 'number') sendPayload.flags = payload.flags
    if (typeof payload.tts === 'boolean') sendPayload.tts = payload.tts
    if (payload.message_reference?.message_id) {
      sendPayload.messageId = payload.message_reference.message_id
    }

    await ops.op_send_message(sendPayload)

    this.#syntheticId += 1
    const id = (Date.now() + this.#syntheticId).toString()
    return buildApiMessage({
      id,
      channelId,
      content: content ?? ''
    })
  }

  private async editChannelMessage(
    channelId: string,
    messageId: string,
    body: unknown
  ) {
    const payload = unwrapBody(body)
    const ops = getCoreOps()

    const editPayload: Record<string, unknown> = {
      channelId,
      messageId
    }
    const content = typeof payload.content === 'string'
      ? payload.content
      : undefined
    if (content !== undefined) editPayload.content = content
    if (Array.isArray(payload.embeds)) editPayload.embeds = payload.embeds
    if (Array.isArray(payload.components)) {
      editPayload.components = payload.components
    }
    if (payload.allowed_mentions !== undefined) {
      editPayload.allowedMentions = payload.allowed_mentions
    }
    if (typeof payload.flags === 'number') editPayload.flags = payload.flags

    await ops.op_edit_message(editPayload)

    return buildApiMessage({
      id: messageId,
      channelId,
      content: content ?? ''
    })
  }
}

function serializeSearchParam(value: unknown): string | null {
  switch (typeof value) {
    case 'string':
      return value
    case 'number':
    case 'bigint':
    case 'boolean':
      return value.toString()
    case 'object':
      if (value === null) return null
      if (value instanceof Date) {
        return Number.isNaN(value.getTime()) ? null : value.toISOString()
      }
      if (
        typeof value.toString === 'function' &&
        value.toString !== Object.prototype.toString
      ) {
        return value.toString()
      }
      return null
    default:
      return null
  }
}

function normalizeRoute(route: string): string {
  if (route.startsWith('http://') || route.startsWith('https://')) {
    return new URL(route).pathname
  }

  const [path = ''] = route.split('?')
  return path.startsWith('/') ? path : `/${path}`
}

function unwrapBody(body: unknown): RestRouteBody {
  if (!body || typeof body !== 'object') return {}

  const maybePayload = body as { body?: unknown }
  if (maybePayload.body && typeof maybePayload.body === 'object') {
    return maybePayload.body as RestRouteBody
  }

  return body as RestRouteBody
}

type FloraCoreOps = {
  op_send_message(args: Record<string, unknown>): Promise<unknown> | unknown
  op_edit_message(args: Record<string, unknown>): Promise<unknown> | unknown
}

function getCoreOps(): FloraCoreOps {
  const denoGlobal = globalThis as {
    Deno?: {
      core?: {
        ops?: Partial<FloraCoreOps>
      }
    }
  }

  const ops = denoGlobal.Deno?.core?.ops
  if (!ops || !ops.op_send_message || !ops.op_edit_message) {
    throw new Error('Deno core ops unavailable in rest polyfill')
  }

  return {
    op_send_message: ops.op_send_message,
    op_edit_message: ops.op_edit_message
  }
}

function buildApiMessage(options: {
  id: string
  channelId: string
  content: string
}) {
  return {
    id: options.id,
    channel_id: options.channelId,
    guild_id: null,
    type: 0,
    content: options.content,
    timestamp: new Date().toISOString(),
    edited_timestamp: null,
    tts: false,
    mention_everyone: false,
    mentions: [],
    mention_roles: [],
    attachments: [],
    embeds: [],
    pinned: false,
    flags: 0,
    author: {
      id: '0',
      username: 'flora',
      discriminator: '0',
      bot: true,
      avatar: null,
      global_name: 'flora'
    }
  }
}
