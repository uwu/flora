export type EmbedField = {
  name: string
  value: string
  inline?: boolean
}

export type Embed = {
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

export class EmbedBuilder {
  #embed: Embed

  constructor(initial: Embed = {}) {
    this.#embed = { ...initial }
  }

  setTitle(title: string) {
    this.#embed.title = title
    return this
  }

  setDescription(description: string) {
    this.#embed.description = description
    return this
  }

  setUrl(url: string) {
    this.#embed.url = url
    return this
  }

  setColor(color: number) {
    this.#embed.color = color
    return this
  }

  setTimestamp(timestamp: string) {
    this.#embed.timestamp = timestamp
    return this
  }

  setFooter(text: string, iconUrl?: string) {
    this.#embed.footer = { text, iconUrl }
    return this
  }

  setImage(url: string) {
    this.#embed.image = { url }
    return this
  }

  setThumbnail(url: string) {
    this.#embed.thumbnail = { url }
    return this
  }

  setAuthor(name?: string, options?: { url?: string; iconUrl?: string }) {
    this.#embed.author = { name, ...options }
    return this
  }

  addField(name: string, value: string, inline = false) {
    const field: EmbedField = { name, value, inline }
    this.#embed.fields = [...(this.#embed.fields ?? []), field]
    return this
  }

  addFields(fields: EmbedField[]) {
    this.#embed.fields = [...(this.#embed.fields ?? []), ...fields]
    return this
  }

  setFields(fields: EmbedField[]) {
    this.#embed.fields = [...fields]
    return this
  }

  toJSON(): Embed {
    return { ...this.#embed }
  }
}

export function embed(initial?: Embed) {
  return new EmbedBuilder(initial)
}

export type Attachment =
  | { url: string; filename?: string; description?: string }
  | { data: string; filename: string; description?: string }

export type AllowedMentions = {
  parse?: Array<'everyone' | 'roles' | 'users'>
  users?: string[]
  roles?: string[]
  repliedUser?: boolean
}

export type MessageReplyOptions = {
  content?: string
  embeds?: Embed[]
  attachments?: Attachment[]
  tts?: boolean
  allowedMentions?: AllowedMentions
  replyTo?: string | null
  ephemeral?: boolean
  flags?: number
}

export type MessageEditOptions = {
  content?: string
  embeds?: Embed[]
  allowedMentions?: AllowedMentions
  flags?: number
}

type BaseContext<TPayload> = {
  msg: TPayload
  reply: (content: string | MessageReplyOptions) => Promise<void>
  edit: (content: string | MessageEditOptions) => Promise<void>
}

export type MessageAuthor = {
  id: string
  username: string
  discriminator?: number | null
  bot: boolean
}

export type MessagePayload = {
  id: string
  channel_id: string
  guild_id?: string | null
  content: string
  author: MessageAuthor
  member?: { roles: string[] } | null
}

export type MessageContext = BaseContext<MessagePayload>

export type MessageUpdatePayload = {
  id: string
  channel_id: string
  guild_id?: string | null
  content?: string | null
  author?: MessageAuthor | null
  edited_timestamp?: string | null
  old?: MessagePayload | null
  new?: MessagePayload | null
}

export type MessageUpdateContext = BaseContext<MessageUpdatePayload>

export type MessageDeletePayload = {
  id: string
  channel_id: string
  guild_id?: string | null
}

export type MessageDeleteContext = BaseContext<MessageDeletePayload>

export type MessageDeleteBulkPayload = {
  ids: string[]
  channel_id: string
  guild_id?: string | null
}

export type MessageDeleteBulkContext = BaseContext<MessageDeleteBulkPayload>

export type InteractionPayload = {
  interaction_id: string
  interaction_token: string
  application_id: string
  guild_id?: string | null
  channel_id?: string | null
  user: MessageAuthor
  command_name: string
  data: any
  locale?: string | null
  guild_locale?: string | null
}
export type SlashCommandOptions = Record<string, string | number | boolean | SlashCommandOptions>

export type InteractionContext = BaseContext<InteractionPayload> & {
  options: SlashCommandOptions
}

export type Command = {
  name: string
  description?: string
  run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
}

export function defineCommand(command: Command): Command {
  return command
}

export type SlashCommand = {
  name: string
  description?: string
  options?: SlashCommandOption[]
  run: (ctx: InteractionContext) => Promise<void> | void
}

export type SlashCommandOption = {
  name: string
  description: string
  type?: 'string' | 'integer' | 'number' | 'boolean'
  required?: boolean
}

export function defineSlashCommand(command: SlashCommand): SlashCommand {
  return command
}

type CreateOptions = {
  prefix?: string
  commands?: Command[]
  prefixCommands?: Command[]
  slashCommands?: SlashCommand[]
}

export function createBot(options: CreateOptions) {
  const prefix = options.prefix ?? '!'
  const commands = options.commands ?? options.prefixCommands ?? []
  const slashCommands = options.slashCommands ?? []

  on('messageCreate', async (ctx: MessageContext) => {
    if (!ctx.msg || !ctx.msg.content) return
    if (ctx.msg.author?.bot) return

    const content = ctx.msg.content.trim()
    if (!content.startsWith(prefix)) return

    const body = content.slice(prefix.length).trim()
    const [commandName, ...args] = body.split(/\s+/)
    const command = commands.find((cmd) => cmd.name === commandName)
    if (!command) return

    await command.run({ ...ctx, args })
  })

  on('interactionCreate', async (ctx: InteractionContext) => {
    if (!ctx.msg) return
    const command = slashCommands.find((cmd) => cmd.name === ctx.msg.command_name)
    if (!command) return

    await command.run(ctx)
  })

  if (slashCommands.length && typeof registerSlashCommands === 'function') {
    // Fire and forget; runtime op will register for this guild isolate only.
    registerSlashCommands(
      slashCommands.map((cmd) => ({
        name: cmd.name,
        description: cmd.description,
        options: cmd.options,
      }))
    )
  }
}
