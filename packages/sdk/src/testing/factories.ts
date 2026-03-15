import type {
  EventComponentInteraction,
  EventInteractionCreate,
  EventMessage,
  EventModalSubmit,
  EventReaction,
  EventReady,
  EventUser
} from '../generated'
import { nextId } from './id'
import type {
  ComponentInteractionPartial,
  DeepPartial,
  InteractionOptions,
  InteractionPartial,
  MessagePartial,
  ModalSubmitPartial,
  ReactionPartial,
  ReadyPartial
} from './types'

function deepMerge<T>(defaults: T, partial?: DeepPartial<T>): T {
  if (!partial) return defaults
  if (typeof defaults !== 'object' || defaults === null) return (partial ?? defaults) as T
  if (Array.isArray(defaults)) return (partial ?? defaults) as T

  const result = { ...defaults } as Record<string, unknown>
  for (const key of Object.keys(partial as object)) {
    const pVal = (partial as Record<string, unknown>)[key]
    const dVal = (defaults as Record<string, unknown>)[key]
    if (
      pVal !== undefined &&
      typeof pVal === 'object' &&
      pVal !== null &&
      !Array.isArray(pVal) &&
      typeof dVal === 'object' &&
      dVal !== null &&
      !Array.isArray(dVal)
    ) {
      result[key] = deepMerge(dVal, pVal as DeepPartial<typeof dVal>)
    } else if (pVal !== undefined) {
      result[key] = pVal
    }
  }
  return result as T
}

export function makeUser(partial?: DeepPartial<EventUser>): EventUser {
  return deepMerge<EventUser>({ id: nextId(), username: 'testuser', bot: false }, partial)
}

export function makeMember(
  partial?: DeepPartial<EventMessage['member']>
): NonNullable<EventMessage['member']> {
  const user = makeUser(partial?.user)
  return deepMerge(
    {
      user,
      roles: [] as string[],
      deaf: false,
      mute: false,
      flags: 0,
      pending: false
    },
    { ...partial, user } as any
  )
}

export function makeMessage(partial?: MessagePartial, guildId?: string): EventMessage {
  const author = makeUser(partial?.author)
  const defaults: EventMessage = {
    id: nextId(),
    channelId: nextId(),
    guildId: guildId ?? nextId(),
    content: '',
    author
  }
  return deepMerge(defaults, partial)
}

function optionToDiscord(name: string, value: string | number | boolean) {
  if (typeof value === 'string') return { name, value, type: 3 }
  if (typeof value === 'boolean') return { name, value, type: 5 }
  if (typeof value === 'number') {
    return Number.isInteger(value) ? { name, value, type: 4 } : { name, value, type: 10 }
  }
  return { name, value }
}

export function makeInteraction(
  name: string,
  opts?: InteractionOptions,
  partial?: InteractionPartial,
  guildId?: string
): EventInteractionCreate {
  const discordOptions = opts ? Object.entries(opts).map(([k, v]) => optionToDiscord(k, v)) : []

  const user = makeUser(partial?.user)
  const defaults: EventInteractionCreate = {
    interactionId: nextId(),
    interactionToken: `test-token-${nextId()}`,
    applicationId: nextId(),
    guildId: guildId ?? nextId(),
    channelId: nextId(),
    user,
    commandName: name,
    data: { options: discordOptions },
    locale: 'en-US'
  }
  return deepMerge(defaults, partial)
}

export function makeComponentInteraction(
  customId: string,
  partial?: ComponentInteractionPartial,
  guildId?: string
): EventComponentInteraction {
  const user = makeUser(partial?.user)
  const defaults: EventComponentInteraction = {
    interactionId: nextId(),
    interactionToken: `test-token-${nextId()}`,
    applicationId: nextId(),
    guildId: guildId ?? nextId(),
    channelId: nextId(),
    user,
    data: { custom_id: customId, component_type: 2 }
  }
  return deepMerge(defaults, partial)
}

export function makeModalSubmit(
  customId: string,
  fields?: Record<string, string>,
  partial?: ModalSubmitPartial,
  guildId?: string
): EventModalSubmit {
  const components = fields
    ? Object.entries(fields).map(([id, value]) => ({
        type: 1,
        components: [{ type: 4, custom_id: id, value }]
      }))
    : []

  const user = makeUser(partial?.user)
  const defaults: EventModalSubmit = {
    interactionId: nextId(),
    interactionToken: `test-token-${nextId()}`,
    applicationId: nextId(),
    guildId: guildId ?? nextId(),
    channelId: nextId(),
    user,
    data: { custom_id: customId, components }
  }
  return deepMerge(defaults, partial)
}

export function makeReaction(partial?: ReactionPartial, guildId?: string): EventReaction {
  const defaults: EventReaction = {
    userId: nextId(),
    channelId: nextId(),
    messageId: nextId(),
    guildId: guildId ?? nextId(),
    emoji: { name: '👍' },
    burst: false
  }
  return deepMerge(defaults, partial)
}

export function makeReady(partial?: ReadyPartial): EventReady {
  const user = makeUser({ bot: true, ...partial?.user })
  const defaults: EventReady = {
    user,
    guildIds: []
  }
  return deepMerge(defaults, partial)
}
