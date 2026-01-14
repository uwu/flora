// #region src/runtime/index.ts
const core = Deno.core
globalThis.__floraHandlers = {}
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
globalThis.console = { log: (...args) => core.ops.op_log(args) }
globalThis.registerSlashCommands = function registerSlashCommands(commands) {
  if (!globalThis.__floraGuildId) return
  return core.ops.op_upsert_guild_commands({
    guildId: globalThis.__floraGuildId,
    commands
  })
}
const CRON_EVENT_PREFIX = '__cron:'
globalThis.cron = function cron(name, cronExpr, handler) {
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
    expr: cronExpr
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

// #endregion
