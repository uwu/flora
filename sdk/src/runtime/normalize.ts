import type { MessageEditOptions, MessageReplyOptions } from '../sdk/types'

export type AnyPayload = {
  id?: string
  messageId?: string
  channelId?: string
  interactionId?: string
  interactionToken?: string
  token?: string
  [key: string]: unknown
}

export function normalizeReply(
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

export function normalizeEdit(
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

export function normalizeInteractionReply(
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
