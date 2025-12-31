type MessageAuthor = {
  id: string
  username: string
  discriminator?: number | null
  bot: boolean
}

type GuildMember = {
  user: MessageAuthor
  nick?: string | null
  avatar?: string | null
  roles: string[]
  joined_at?: string | null
  premium_since?: string | null
  deaf: boolean
  mute: boolean
  flags: number
  pending: boolean
  permissions?: string | null
  communication_disabled_until?: string | null
}

type EmbedField = {
  name: string
  value: string
  inline?: boolean
}

type Embed = {
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

type Attachment =
  | { url: string; filename?: string; description?: string }
  | { data: string; filename: string; description?: string }

type AllowedMentions = {
  parse?: Array<'everyone' | 'roles' | 'users'>
  users?: string[]
  roles?: string[]
  repliedUser?: boolean
}

type MessageReplyOptions = {
  content?: string
  embeds?: Embed[]
  attachments?: Attachment[]
  tts?: boolean
  allowedMentions?: AllowedMentions
  replyTo?: string | null
  ephemeral?: boolean
  flags?: number
}

type MessageEditOptions = {
  content?: string
  embeds?: Embed[]
  allowedMentions?: AllowedMentions
  flags?: number
}

type SlashCommandOptions = Record<string, string | number | boolean | SlashCommandOptions>

declare global {
  type EmbedBuilder
  type MessageAuthor
  type GuildMember
  type Embed
  type EmbedField
  type Attachment
  type AllowedMentions
  type MessageReplyOptions
  type MessageEditOptions
  type SlashCommandOptions

  type MessageContext
  type MessageUpdateContext
  type MessageDeleteContext
  type MessageDeleteBulkContext
  type InteractionContext
  type Command
  type SlashCommand
  type SlashCommandOption

  const EmbedBuilder: new (initial?: Embed) => EmbedBuilder
  const embed: (initial?: Embed) => EmbedBuilder
  const on: (
    event: 'messageCreate',
    handler: (ctx: MessageContext) => void | Promise<void>
  ) => void
  const on: (
    event: 'messageUpdate',
    handler: (ctx: MessageUpdateContext) => void | Promise<void>
  ) => void
  const on: (
    event: 'messageDelete',
    handler: (ctx: MessageDeleteContext) => void | Promise<void>
  ) => void
  const on: (
    event: 'messageDeleteBulk',
    handler: (ctx: MessageDeleteBulkContext) => void | Promise<void>
  ) => void
  const on: (
    event: 'interactionCreate',
    handler: (ctx: InteractionContext) => void | Promise<void>
  ) => void

  const createBot: (options: {
    prefix?: string
    commands?: Command[]
    prefixCommands?: Command[]
    slashCommands?: SlashCommand[]
  }) => void
  const defineCommand: (command: {
    name: string
    description?: string
    run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
  }) => Command
  const defineSlashCommand: (command: {
    name: string
    description: string
    options?: SlashCommandOption[]
    subcommands?: Array<{
      name: string
      description: string
      options?: SlashCommandOption[]
      run: (ctx: InteractionContext) => Promise<void> | void
    }>
    run?: (ctx: InteractionContext) => Promise<void> | void
  }) => SlashCommand
  const hasRole: (ctx: InteractionContext, roleId: string) => boolean
  const getSubcommand: (ctx: InteractionContext) => string | undefined
  const getSubcommandGroup: (ctx: InteractionContext) => string | undefined
  const kv: {
    store: (name: string) => {
      get: (key: string) => Promise<string | null>
      set: (key: string, value: string, options?: {
        expiration?: number
        metadata?: Record<string, unknown>
      }) => Promise<void>
      delete: (key: string) => Promise<void>
      list: (options?: {
        prefix?: string
        limit?: number
        cursor?: string
      }) => Promise<{
        keys: Array<{
          name: string
          expiration?: number
          metadata?: Record<string, unknown>
        }>
        list_complete: boolean
        cursor: string | null
      }>
    }
  }

  export {}
}

export {}
