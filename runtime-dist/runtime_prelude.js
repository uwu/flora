// #region src/runtime/index.ts
const core = Deno.core
const maybeBuffer = globalThis.Buffer
if (typeof globalThis.btoa === 'undefined') {
  globalThis.btoa = (input) =>
    maybeBuffer ? maybeBuffer.from(input, 'binary').toString('base64') : String(input)
}
if (typeof globalThis.atob === 'undefined') {
  globalThis.atob = (input) =>
    maybeBuffer ? maybeBuffer.from(input, 'base64').toString('binary') : String(input)
}
if (typeof globalThis.TextEncoder === 'undefined') {
  class TextEncoderPolyfill {
    encode(input) {
      if (maybeBuffer) {
        return Uint8Array.from(
          maybeBuffer.from(input, 'utf8').toString('binary'),
          (char) => char.charCodeAt(0)
        )
      }
      const bytes = []
      for (let i = 0; i < input.length; i++) {
        bytes.push(input.charCodeAt(i) & 255)
      }
      return Uint8Array.from(bytes)
    }
  }
  globalThis.TextEncoder = TextEncoderPolyfill
}
if (typeof globalThis.TextDecoder === 'undefined') {
  class TextDecoderPolyfill {
    decode(input) {
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
  globalThis.TextDecoder = TextDecoderPolyfill
}
if (typeof globalThis.Blob === 'undefined') {
  class BlobPolyfill {
    chunks
    size
    type
    constructor(parts = [], options = {}) {
      this.chunks = parts.map(toUint8)
      this.size = this.chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0)
      this.type = options.type ?? ''
    }
    async arrayBuffer() {
      return concatChunks(this.chunks).buffer
    }
    async text() {
      return new TextDecoder().decode(concatChunks(this.chunks))
    }
  }
  globalThis.Blob = BlobPolyfill
}
globalThis.__floraHandlers = {}
globalThis.secrets = {
  get(name) {
    return core.ops.op_secret_placeholder(name)
  }
}
globalThis.on = function on(event, handler) {
  if (!globalThis.__floraHandlers[event]) {
    globalThis.__floraHandlers[event] = []
  }
  globalThis.__floraHandlers[event].push(handler)
}
globalThis.__floraDispatch = async function __floraDispatch(event, payload) {
  const handlers = globalThis.__floraHandlers[event] || []
  for (const handler of handlers) {
    const context = {
      msg: payload,
      reply(message) {
        const options = normalizeReply(message, payload)
        if (options['interactionId'] && options['token']) {
          return core.ops.op_send_interaction_response(options)
        }
        return core.ops.op_send_message(options)
      },
      edit(message) {
        const options = normalizeEdit(message, payload)
        return core.ops.op_edit_message(options)
      }
    }
    await handler(context)
  }
}
globalThis.console = {
  log: (...args) => core.ops.op_log(args),
  warn: (...args) => core.ops.op_log(args)
}
globalThis.registerSlashCommands = function registerSlashCommands(commands) {
  if (!globalThis.__floraGuildId) return
  return core.ops.op_upsert_guild_commands({
    guildId: globalThis.__floraGuildId,
    commands
  })
}
const CRON_EVENT_PREFIX = '__cron:'
globalThis.cron = function cron(name, cronExpr, handler, options) {
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
function normalizeReply(message, payload) {
  if (payload?.interactionToken) {
    return normalizeInteractionReply(message, payload)
  }
  const base = { channelId: payload.channelId }
  const replyId = payload.id ?? payload.messageId
  if (typeof message === 'string') {
    return {
      ...base,
      messageId: replyId,
      content: message
    }
  }
  if (message && typeof message === 'object') {
    const normalized = {
      ...base,
      ...message
    }
    const explicitReplyTo = message.replyTo ?? message.replyTo
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
  return {
    ...base,
    messageId: replyId,
    content: String(message)
  }
}
function normalizeEdit(message, payload) {
  const messageId = payload.id ?? payload.messageId
  if (!messageId || !payload?.channelId) {
    throw new Error('Message edit requires a message payload')
  }
  const base = {
    channelId: payload.channelId,
    messageId
  }
  if (typeof message === 'string') {
    return {
      ...base,
      content: message
    }
  }
  if (message && typeof message === 'object') {
    return {
      ...base,
      ...message
    }
  }
  return {
    ...base,
    content: String(message)
  }
}
function normalizeInteractionReply(message, payload) {
  const base = {
    interactionId: payload.interactionId ?? payload.id,
    token: payload.interactionToken
  }
  if (typeof message === 'string') {
    return {
      ...base,
      content: message
    }
  }
  if (message && typeof message === 'object') {
    const normalized = {
      ...base,
      ...message
    }
    if (message.ephemeral !== undefined) {
      normalized.ephemeral = message.ephemeral
    }
    return normalized
  }
  return {
    ...base,
    content: String(message)
  }
}
function toUint8(part) {
  if (typeof part === 'string') {
    return new TextEncoder().encode(part)
  }
  if (part instanceof ArrayBuffer) {
    return new Uint8Array(part)
  }
  return new Uint8Array(part.buffer, part.byteOffset, part.byteLength)
}
function concatChunks(chunks) {
  const total = chunks.reduce((sum, chunk) => sum + chunk.byteLength, 0)
  const out = new Uint8Array(total)
  let offset = 0
  for (const chunk of chunks) {
    out.set(chunk, offset)
    offset += chunk.byteLength
  }
  return out
}

// #endregion
