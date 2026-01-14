// Auto-generated global types for Flora SDK
// Do not edit manually - regenerate with `bun run build`

declare global {
  // Runtime exports (from runtime/index.ts)
  interface FloraEventMap {
    ready: {
      msg: EventReady
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    messageCreate: {
      msg: EventMessage
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    messageUpdate: {
      msg: EventMessageUpdate
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    messageDelete: {
      msg: EventMessageDelete
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    messageDeleteBulk: {
      msg: EventMessageDeleteBulk
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    interactionCreate: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions }
    componentInteraction: {
      msg: EventComponentInteraction
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    modalSubmit: {
      msg: EventModalSubmit
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    reactionAdd: {
      msg: EventReaction
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    reactionRemove: {
      msg: EventReaction
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    reactionRemoveEmoji: {
      msg: EventReaction
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
    reactionRemoveAll: {
      msg: EventReactionRemoveAll
      reply: (content: string | MessageReplyOptions) => Promise<void>
      edit: (content: string | MessageEditOptions) => Promise<void>
    }
  }

  type FloraEventHandler<E extends keyof FloraEventMap> = (
    ctx: FloraEventMap[E]
  ) => void | Promise<void>

  interface CronContext {
    name: string
    scheduledAt: string
  }

  interface CronOptions {
    skipIfRunning?: boolean | undefined
  }

  type CronHandler = (ctx: CronContext) => void | Promise<void>

  var __floraHandlers: { [x: string]: Array<Function> }

  var __floraGuildId: string | undefined

  function on<E extends keyof FloraEventMap>(
    event: E,
    handler: (ctx: FloraEventMap[E]) => void | Promise<void>
  ): void

  function __floraDispatch(event: string, payload: unknown): Promise<void>

  function registerSlashCommands(commands: Array<FlattenedSlashCommand>): Promise<void> | undefined

  function cron(
    name: string,
    cronExpr: string,
    handler: (ctx: CronContext) => void | Promise<void>,
    options?: CronOptions | undefined
  ): void

  // SDK exports (functions, consts, classes, types)
  function prefix(
    command: {
      name: string
      description?: string
      run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
    }
  ): {
    name: string
    description?: string
    run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
  }

  function slash(
    command: {
      name: string
      description: string
      options?: SlashCommandOption[]
      subcommands?: SlashSubcommand[]
      run?: (ctx: InteractionContext) => Promise<void> | void
    }
  ): {
    name: string
    description: string
    options?: SlashCommandOption[]
    subcommands?: SlashSubcommand[]
    run?: (ctx: InteractionContext) => Promise<void> | void
  }

  function createBot(
    options: {
      prefix?: string
      commands?: Command[]
      prefixCommands?: Command[]
      slashCommands?: SlashCommand[]
    }
  ): void

  function flattenCommands(commands: Array<SlashCommand>): Array<FlattenedSlashCommand>

  function handleSubcommand(
    ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions },
    command: {
      name: string
      description: string
      options?: SlashCommandOption[]
      subcommands?: SlashSubcommand[]
      run?: (ctx: InteractionContext) => Promise<void> | void
    }
  ): Promise<void>

  function flattenInteractionOptions(options: Array<any>): { [x: string]: any }

  type Command = {
    name: string
    description?: string
    run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
  }

  type SlashCommandOption = {
    name: string
    description: string
    type?: 'string' | 'integer' | 'number' | 'boolean' | 'subcommand' | 'subcommand_group'
    required?: boolean
    options?: SlashCommandOption[]
  }

  type SlashSubcommand = {
    name: string
    description: string
    options?: SlashCommandOption[]
    run: (ctx: InteractionContext) => Promise<void> | void
  }

  type SlashCommand = {
    name: string
    description: string
    options?: SlashCommandOption[]
    subcommands?: SlashSubcommand[]
    run?: (ctx: InteractionContext) => Promise<void> | void
  }

  type CreateOptions = {
    prefix?: string
    commands?: Command[]
    prefixCommands?: Command[]
    slashCommands?: SlashCommand[]
  }

  type FlattenedSlashCommand = { name: string; description: string; options?: SlashCommandOption[] }

  type SubcommandMap = {
    [x: string]: Record<string, (ctx: InteractionContext) => Promise<void> | void>
  }

  function embed(initial?: Embed | undefined): EmbedBuilder

  class EmbedBuilder {
    setTitle(title: string): this
    setDescription(description: string): this
    setUrl(url: string): this
    setColor(color: number): this
    setTimestamp(timestamp: string): this
    setFooter(text: string, iconUrl?: string | undefined): this
    setImage(url: string): this
    setThumbnail(url: string): this
    setAuthor(
      name?: string | undefined,
      options?: { url?: string; iconUrl?: string } | undefined
    ): this
    addField(name: string, value: string, inline?: boolean): this
    addFields(fields: Array<EmbedField>): this
    setFields(fields: Array<EmbedField>): this
    toJSON(): {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }
  }

  function hasRole(
    ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions },
    roleId: string
  ): boolean

  function getSubcommand(
    ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions }
  ): string | undefined

  function getSubcommandGroup(
    ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions }
  ): string | undefined

  function store(name: string): KvStore

  type RawKvGetResult = { value: string | null; metadata?: Record<string, unknown> }

  class KvStore {
    get(key: string): Promise<string | null>
    getWithMetadata(key: string): Promise<RawKvGetResult>
    set(key: string, value: string, options?: RawKvSetOptions | undefined): Promise<void>
    updateMetadata(key: string, metadata: JsonValue | undefined): Promise<void>
    delete(key: string): Promise<void>
    list(options?: RawKvListKeysOptions | undefined): Promise<RawKvListKeysResult>
  }

  const kv: { store: (name: string) => KvStore }

  const rest: {
    sendMessage: (args: RawSendMessage) => Promise<void>
    editMessage: (args: RawEditMessage) => Promise<void>
    deleteMessage: (args: RawDeleteMessage) => Promise<void>
    bulkDeleteMessages: (args: RawBulkDeleteMessages) => Promise<void>
    pinMessage: (args: RawPinMessage) => Promise<void>
    unpinMessage: (args: RawPinMessage) => Promise<void>
    crosspostMessage: (args: RawCrosspostMessage) => Promise<JsonValue>
    fetchMessage: (args: RawFetchMessage) => Promise<JsonValue>
    fetchMessages: (args: RawFetchMessages) => Promise<JsonValue[]>
    addReaction: (args: RawReaction) => Promise<void>
    removeReaction: (args: RawReaction) => Promise<void>
    clearReactions: (args: RawClearReactions) => Promise<void>
    sendInteractionResponse: (args: RawInteractionResponse) => Promise<void>
    deferInteractionResponse: (args: RawDeferInteractionResponse) => Promise<void>
    updateInteractionResponse: (args: RawUpdateInteractionResponse) => Promise<void>
    editOriginalInteractionResponse: (args: RawEditInteractionResponse) => Promise<JsonValue>
    deleteOriginalInteractionResponse: (args: RawDeleteInteractionResponse) => Promise<void>
    createFollowupMessage: (args: RawFollowupMessage) => Promise<JsonValue>
    editFollowupMessage: (args: RawFollowupMessage) => Promise<JsonValue>
    deleteFollowupMessage: (args: RawDeleteFollowupMessage) => Promise<void>
    upsertGuildCommands: (args: RawUpsertGuildCommands) => Promise<void>
    createGuildCommand: (args: RawCreateGuildCommand) => Promise<JsonValue>
    editGuildCommand: (args: RawEditGuildCommand) => Promise<JsonValue>
    deleteGuildCommand: (args: RawDeleteGuildCommand) => Promise<void>
    getGuildCommands: (args: RawGuildId) => Promise<JsonValue[]>
    getGuildCommand: (args: RawGetGuildCommand) => Promise<JsonValue>
    editGuildCommandPermissions: (args: RawCommandPermissions) => Promise<JsonValue>
    getGuildCommandsPermissions: (args: RawGuildId) => Promise<JsonValue[]>
    getGuildCommandPermissions: (args: RawGetGuildCommand) => Promise<JsonValue>
    kickMember: (args: RawGuildUser) => Promise<void>
    banMember: (args: RawBanMember) => Promise<void>
    unbanMember: (args: RawGuildUser) => Promise<void>
    addMemberRole: (args: RawMemberRole) => Promise<void>
    removeMemberRole: (args: RawMemberRole) => Promise<void>
    editMember: (args: RawEditMember) => Promise<JsonValue>
    createChannel: (args: RawCreateChannel) => Promise<JsonValue>
    editChannel: (args: RawEditChannel) => Promise<JsonValue>
    deleteChannel: (args: RawDeleteChannel) => Promise<JsonValue>
    createThread: (args: RawCreateThread) => Promise<JsonValue>
    createThreadFromMessage: (args: RawCreateThreadFromMessage) => Promise<JsonValue>
    joinThread: (args: RawThreadId) => Promise<void>
    leaveThread: (args: RawThreadId) => Promise<void>
    addThreadMember: (args: RawThreadMember) => Promise<void>
    removeThreadMember: (args: RawThreadMember) => Promise<void>
    executeWebhook: (args: RawExecuteWebhook) => Promise<JsonValue | null>
    editWebhook: (args: RawEditWebhook) => Promise<JsonValue>
    deleteWebhook: (args: RawDeleteWebhook) => Promise<void>
  }

  type Embed = {
    title?: string
    description?: string
    url?: string
    color?: number
    timestamp?: string
    footer?: { text?: string; iconUrl?: string }
    image?: { url?: string }
    thumbnail?: { url?: string }
    author?: { name?: string; url?: string; iconUrl?: string }
    fields?: { name: string; value: string; inline: boolean }[]
  }

  type EmbedField = { name: string; value: string; inline: boolean }

  type MessageReplyOptions = {
    content?: string
    embeds?: RawEmbed[]
    attachments?: RawAttachment[]
    components?: JsonValue[]
    tts?: boolean
    allowedMentions?: RawAllowedMentions
    replyTo?: string | null
    ephemeral?: boolean
    flags?: number
  }

  type MessageEditOptions = {
    content?: string
    embeds?: RawEmbed[]
    components?: JsonValue[]
    allowedMentions?: RawAllowedMentions
    flags?: number
  }

  type BaseContext<TPayload> = {
    msg: TPayload
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type MessageContext = {
    msg: EventMessage
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type MessageUpdateContext = {
    msg: EventMessageUpdate
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type MessageDeleteContext = {
    msg: EventMessageDelete
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type MessageDeleteBulkContext = {
    msg: EventMessageDeleteBulk
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type ComponentInteractionContext = {
    msg: EventComponentInteraction
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type ModalSubmitContext = {
    msg: EventModalSubmit
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type ReactionContext = {
    msg: EventReaction
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type ReactionRemoveAllContext = {
    msg: EventReactionRemoveAll
    reply: (content: string | MessageReplyOptions) => Promise<void>
    edit: (content: string | MessageEditOptions) => Promise<void>
  }

  type SlashCommandOptions = { [x: string]: string | number | boolean | undefined }

  type InteractionContext = BaseContext<EventInteractionCreate> & { options: SlashCommandOptions }

  type JsonValue = string | number | boolean | Array<JsonValue> | {
    [x: string]: JsonValue | undefined
  } | null

  type EventUser = { id: string; username: string; discriminator?: number; bot: boolean }

  type EventMember = {
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    nick?: string
    avatar?: string
    roles: string[]
    joinedAt?: string
    premiumSince?: string
    deaf: boolean
    mute: boolean
    flags: number
    pending: boolean
    permissions?: string
    communicationDisabledUntil?: string
  }

  type EventMessage = {
    id: string
    channelId: string
    guildId?: string
    content: string
    author: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joinedAt?: string
      premiumSince?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communicationDisabledUntil?: string
    }
  }

  type EventMessageUpdate = {
    id: string
    channelId: string
    guildId?: string
    content?: string
    author?: { id: string; username: string; discriminator?: number; bot: boolean }
    editedTimestamp?: string
    old?: {
      id: string
      channelId: string
      guildId?: string
      content: string
      author: { id: string; username: string; discriminator?: number; bot: boolean }
      member?: {
        user: { id: string; username: string; discriminator?: number; bot: boolean }
        nick?: string
        avatar?: string
        roles: string[]
        joinedAt?: string
        premiumSince?: string
        deaf: boolean
        mute: boolean
        flags: number
        pending: boolean
        permissions?: string
        communicationDisabledUntil?: string
      }
    }
    new?: {
      id: string
      channelId: string
      guildId?: string
      content: string
      author: { id: string; username: string; discriminator?: number; bot: boolean }
      member?: {
        user: { id: string; username: string; discriminator?: number; bot: boolean }
        nick?: string
        avatar?: string
        roles: string[]
        joinedAt?: string
        premiumSince?: string
        deaf: boolean
        mute: boolean
        flags: number
        pending: boolean
        permissions?: string
        communicationDisabledUntil?: string
      }
    }
  }

  type EventMessageDelete = { id: string; channelId: string; guildId?: string }

  type EventMessageDeleteBulk = { ids: string[]; channelId: string; guildId?: string }

  type EventReady = {
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    guildIds: string[]
  }

  type EventInteractionCreate = {
    interactionId: string
    interactionToken: string
    applicationId: string
    guildId?: string
    channelId?: string
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joinedAt?: string
      premiumSince?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communicationDisabledUntil?: string
    }
    commandName: string
    data: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    locale?: string
    guildLocale?: string
  }

  type EventComponentInteraction = {
    interactionId: string
    interactionToken: string
    applicationId: string
    guildId?: string
    channelId?: string
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joinedAt?: string
      premiumSince?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communicationDisabledUntil?: string
    }
    data: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    locale?: string
    guildLocale?: string
    messageId?: string
  }

  type EventModalSubmit = {
    interactionId: string
    interactionToken: string
    applicationId: string
    guildId?: string
    channelId?: string
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joinedAt?: string
      premiumSince?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communicationDisabledUntil?: string
    }
    data: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    locale?: string
    guildLocale?: string
    messageId?: string
  }

  type EventReaction = {
    userId?: string
    channelId: string
    messageId: string
    guildId?: string
    emoji: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    burst: boolean
  }

  type EventReactionRemoveAll = { messageId: string; channelId: string; guildId?: string }

  type RawKvKeyMetadata = {
    expiration?: bigint
    metadata?: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
  }

  type RawKvKeyInfo = {
    name: string
    expiration?: bigint
    metadata?: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
  }

  type RawKvListKeysResult = {
    keys: {
      name: string
      expiration?: bigint
      metadata?:
        | number
        | string
        | boolean
        | Array<JsonValue>
        | { [key in string]?: JsonValue }
        | null
    }[]
    listComplete: boolean
    cursor?: string
  }

  type RawKvSetOptions = {
    expiration?: bigint
    metadata?: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
  }

  type RawKvListKeysOptions = { prefix?: string; limit?: bigint; cursor?: string }

  type RawEmbedMedia = { url?: string }

  type RawEmbedFooter = { text?: string; iconUrl?: string }

  type RawEmbedAuthor = { name?: string; url?: string; iconUrl?: string }

  type RawEmbedField = { name: string; value: string; inline: boolean }

  type RawEmbed = {
    title?: string
    description?: string
    url?: string
    color?: number
    timestamp?: string
    footer?: { text?: string; iconUrl?: string }
    image?: { url?: string }
    thumbnail?: { url?: string }
    author?: { name?: string; url?: string; iconUrl?: string }
    fields?: { name: string; value: string; inline: boolean }[]
  }

  type RawSendMessage = {
    channelId: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { url: { url: string; filename?: string; description?: string } } | {
      base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowedMentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      repliedUser?: boolean
    }
    flags?: bigint
    messageId?: string
    replyTo?: string
  }

  type RawEditMessage = {
    channelId: string
    messageId: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    allowedMentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      repliedUser?: boolean
    }
    flags?: bigint
  }

  type RawDeleteMessage = { channelId: string; messageId: string }

  type RawBulkDeleteMessages = { channelId: string; messageIds: string[] }

  type RawPinMessage = { channelId: string; messageId: string }

  type RawReaction = { channelId: string; messageId: string; emoji: string; userId?: string }

  type RawClearReactions = { channelId: string; messageId: string; emoji?: string }

  type RawFetchMessage = { channelId: string; messageId: string }

  type RawFetchMessages = {
    channelId: string
    limit?: number
    before?: string
    after?: string
    around?: string
  }

  type RawCrosspostMessage = { channelId: string; messageId: string }

  type RawInteractionResponse = {
    interactionId: string
    token: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { url: { url: string; filename?: string; description?: string } } | {
      base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowedMentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      repliedUser?: boolean
    }
    ephemeral?: boolean
    flags?: bigint
  }

  type RawUpsertGuildCommands = {
    guildId: string
    commands: {
      name: string
      description?: string
      options?: {
        name: string
        description: string
        kind?: string
        required?: boolean
        options?: RawSlashCommandOption[]
      }[]
    }[]
  }

  type RawSlashCommand = {
    name: string
    description?: string
    options?: {
      name: string
      description: string
      kind?: string
      required?: boolean
      options?: RawSlashCommandOption[]
    }[]
  }

  type RawSlashCommandOption = {
    name: string
    description: string
    kind?: string
    required?: boolean
    options?: RawSlashCommandOption[]
  }

  type RawDeferInteractionResponse = { interactionId: string; token: string; ephemeral?: boolean }

  type RawUpdateInteractionResponse = {
    interactionId: string
    token: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { url: { url: string; filename?: string; description?: string } } | {
      base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowedMentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      repliedUser?: boolean
    }
    flags?: bigint
  }

  type RawEditInteractionResponse = {
    token: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { url: { url: string; filename?: string; description?: string } } | {
      base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    allowedMentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      repliedUser?: boolean
    }
    flags?: bigint
  }

  type RawDeleteInteractionResponse = { token: string }

  type RawFollowupMessage = {
    token: string
    messageId?: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { url: { url: string; filename?: string; description?: string } } | {
      base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowedMentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      repliedUser?: boolean
    }
    flags?: bigint
  }

  type RawDeleteFollowupMessage = { token: string; messageId: string }

  type RawCreateGuildCommand = {
    guildId: string
    command: {
      name: string
      description?: string
      options?: {
        name: string
        description: string
        kind?: string
        required?: boolean
        options?: RawSlashCommandOption[]
      }[]
    }
  }

  type RawEditGuildCommand = {
    guildId: string
    commandId: string
    command: {
      name: string
      description?: string
      options?: {
        name: string
        description: string
        kind?: string
        required?: boolean
        options?: RawSlashCommandOption[]
      }[]
    }
  }

  type RawDeleteGuildCommand = { guildId: string; commandId: string }

  type RawGetGuildCommand = { guildId: string; commandId: string }

  type RawCommandPermissions = {
    guildId: string
    commandId: string
    permissions:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null
  }

  type RawGuildId = { guildId: string }

  type RawGuildUser = { guildId: string; userId: string; reason?: string }

  type RawBanMember = {
    guildId: string
    userId: string
    deleteMessageSeconds?: number
    reason?: string
  }

  type RawMemberRole = { guildId: string; userId: string; roleId: string; reason?: string }

  type RawEditMember = {
    guildId: string
    userId: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawCreateChannel = {
    guildId: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawEditChannel = {
    channelId: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawDeleteChannel = { channelId: string; reason?: string }

  type RawCreateThread = {
    channelId: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawCreateThreadFromMessage = {
    channelId: string
    messageId: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawThreadId = { threadId: string }

  type RawThreadMember = { threadId: string; userId: string }

  type RawExecuteWebhook = {
    webhookId: string
    token: string
    wait?: boolean
    threadId?: string
    withComponents?: boolean
    content?: string
    username?: string
    avatarUrl?: string
    tts?: boolean
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; iconUrl?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; iconUrl?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { url: { url: string; filename?: string; description?: string } } | {
      base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    allowedMentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      repliedUser?: boolean
    }
    flags?: bigint
    threadName?: string
  }

  type RawEditWebhook = {
    webhookId: string
    token?: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawDeleteWebhook = { webhookId: string; token?: string; reason?: string }

  const flora: typeof import('./src/index')
}

export {}
