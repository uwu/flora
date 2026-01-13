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

  var __floraHandlers: { [x: string]: Array<Function> }

  var __floraGuildId: string | undefined

  function on<E extends keyof FloraEventMap>(
    event: E,
    handler: (ctx: FloraEventMap[E]) => void | Promise<void>
  ): void

  function __floraDispatch(event: string, payload: unknown): Promise<void>

  function registerSlashCommands(commands: Array<FlattenedSlashCommand>): Promise<void> | undefined

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
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
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
    footer?: { text?: string; icon_url?: string }
    image?: { url?: string }
    thumbnail?: { url?: string }
    author?: { name?: string; url?: string; icon_url?: string }
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
    joined_at?: string
    premium_since?: string
    deaf: boolean
    mute: boolean
    flags: number
    pending: boolean
    permissions?: string
    communication_disabled_until?: string
  }

  type EventMessage = {
    id: string
    channel_id: string
    guild_id?: string
    content: string
    author: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joined_at?: string
      premium_since?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communication_disabled_until?: string
    }
  }

  type EventMessageUpdate = {
    id: string
    channel_id: string
    guild_id?: string
    content?: string
    author?: { id: string; username: string; discriminator?: number; bot: boolean }
    edited_timestamp?: string
    old?: {
      id: string
      channel_id: string
      guild_id?: string
      content: string
      author: { id: string; username: string; discriminator?: number; bot: boolean }
      member?: {
        user: { id: string; username: string; discriminator?: number; bot: boolean }
        nick?: string
        avatar?: string
        roles: string[]
        joined_at?: string
        premium_since?: string
        deaf: boolean
        mute: boolean
        flags: number
        pending: boolean
        permissions?: string
        communication_disabled_until?: string
      }
    }
    new?: {
      id: string
      channel_id: string
      guild_id?: string
      content: string
      author: { id: string; username: string; discriminator?: number; bot: boolean }
      member?: {
        user: { id: string; username: string; discriminator?: number; bot: boolean }
        nick?: string
        avatar?: string
        roles: string[]
        joined_at?: string
        premium_since?: string
        deaf: boolean
        mute: boolean
        flags: number
        pending: boolean
        permissions?: string
        communication_disabled_until?: string
      }
    }
  }

  type EventMessageDelete = { id: string; channel_id: string; guild_id?: string }

  type EventMessageDeleteBulk = { ids: string[]; channel_id: string; guild_id?: string }

  type EventReady = {
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    guild_ids: string[]
  }

  type EventInteractionCreate = {
    interaction_id: string
    interaction_token: string
    application_id: string
    guild_id?: string
    channel_id?: string
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joined_at?: string
      premium_since?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communication_disabled_until?: string
    }
    command_name: string
    data: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    locale?: string
    guild_locale?: string
  }

  type EventComponentInteraction = {
    interaction_id: string
    interaction_token: string
    application_id: string
    guild_id?: string
    channel_id?: string
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joined_at?: string
      premium_since?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communication_disabled_until?: string
    }
    data: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    locale?: string
    guild_locale?: string
    message_id?: string
  }

  type EventModalSubmit = {
    interaction_id: string
    interaction_token: string
    application_id: string
    guild_id?: string
    channel_id?: string
    user: { id: string; username: string; discriminator?: number; bot: boolean }
    member?: {
      user: { id: string; username: string; discriminator?: number; bot: boolean }
      nick?: string
      avatar?: string
      roles: string[]
      joined_at?: string
      premium_since?: string
      deaf: boolean
      mute: boolean
      flags: number
      pending: boolean
      permissions?: string
      communication_disabled_until?: string
    }
    data: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    locale?: string
    guild_locale?: string
    message_id?: string
  }

  type EventReaction = {
    user_id?: string
    channel_id: string
    message_id: string
    guild_id?: string
    emoji: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    burst: boolean
  }

  type EventReactionRemoveAll = { message_id: string; channel_id: string; guild_id?: string }

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
    list_complete: boolean
    cursor?: string
  }

  type RawKvSetOptions = {
    expiration?: bigint
    metadata?: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
  }

  type RawKvListKeysOptions = { prefix?: string; limit?: bigint; cursor?: string }

  type RawEmbedMedia = { url?: string }

  type RawEmbedFooter = { text?: string; icon_url?: string }

  type RawEmbedAuthor = { name?: string; url?: string; icon_url?: string }

  type RawEmbedField = { name: string; value: string; inline: boolean }

  type RawEmbed = {
    title?: string
    description?: string
    url?: string
    color?: number
    timestamp?: string
    footer?: { text?: string; icon_url?: string }
    image?: { url?: string }
    thumbnail?: { url?: string }
    author?: { name?: string; url?: string; icon_url?: string }
    fields?: { name: string; value: string; inline: boolean }[]
  }

  type RawSendMessage = {
    channel_id: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { Url: { url: string; filename?: string; description?: string } } | {
      Base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowed_mentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      replied_user?: boolean
    }
    flags?: bigint
    message_id?: string
    reply_to?: string
  }

  type RawEditMessage = {
    channel_id: string
    message_id: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    allowed_mentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      replied_user?: boolean
    }
    flags?: bigint
  }

  type RawDeleteMessage = { channel_id: string; message_id: string }

  type RawBulkDeleteMessages = { channel_id: string; message_ids: string[] }

  type RawPinMessage = { channel_id: string; message_id: string }

  type RawReaction = { channel_id: string; message_id: string; emoji: string; user_id?: string }

  type RawClearReactions = { channel_id: string; message_id: string; emoji?: string }

  type RawFetchMessage = { channel_id: string; message_id: string }

  type RawFetchMessages = {
    channel_id: string
    limit?: number
    before?: string
    after?: string
    around?: string
  }

  type RawCrosspostMessage = { channel_id: string; message_id: string }

  type RawInteractionResponse = {
    interaction_id: string
    token: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { Url: { url: string; filename?: string; description?: string } } | {
      Base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowed_mentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      replied_user?: boolean
    }
    ephemeral?: boolean
    flags?: bigint
  }

  type RawUpsertGuildCommands = {
    guild_id: string
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

  type RawDeferInteractionResponse = { interaction_id: string; token: string; ephemeral?: boolean }

  type RawUpdateInteractionResponse = {
    interaction_id: string
    token: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { Url: { url: string; filename?: string; description?: string } } | {
      Base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowed_mentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      replied_user?: boolean
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
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { Url: { url: string; filename?: string; description?: string } } | {
      Base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    allowed_mentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      replied_user?: boolean
    }
    flags?: bigint
  }

  type RawDeleteInteractionResponse = { token: string }

  type RawFollowupMessage = {
    token: string
    message_id?: string
    content?: string
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { Url: { url: string; filename?: string; description?: string } } | {
      Base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    tts?: boolean
    allowed_mentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      replied_user?: boolean
    }
    flags?: bigint
  }

  type RawDeleteFollowupMessage = { token: string; message_id: string }

  type RawCreateGuildCommand = {
    guild_id: string
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
    guild_id: string
    command_id: string
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

  type RawDeleteGuildCommand = { guild_id: string; command_id: string }

  type RawGetGuildCommand = { guild_id: string; command_id: string }

  type RawCommandPermissions = {
    guild_id: string
    command_id: string
    permissions:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null
  }

  type RawGuildId = { guild_id: string }

  type RawGuildUser = { guild_id: string; user_id: string; reason?: string }

  type RawBanMember = {
    guild_id: string
    user_id: string
    delete_message_seconds?: number
    reason?: string
  }

  type RawMemberRole = { guild_id: string; user_id: string; role_id: string; reason?: string }

  type RawEditMember = {
    guild_id: string
    user_id: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawCreateChannel = {
    guild_id: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawEditChannel = {
    channel_id: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawDeleteChannel = { channel_id: string; reason?: string }

  type RawCreateThread = {
    channel_id: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawCreateThreadFromMessage = {
    channel_id: string
    message_id: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawThreadId = { thread_id: string }

  type RawThreadMember = { thread_id: string; user_id: string }

  type RawExecuteWebhook = {
    webhook_id: string
    token: string
    wait?: boolean
    thread_id?: string
    with_components?: boolean
    content?: string
    username?: string
    avatar_url?: string
    tts?: boolean
    embeds?: {
      title?: string
      description?: string
      url?: string
      color?: number
      timestamp?: string
      footer?: { text?: string; icon_url?: string }
      image?: { url?: string }
      thumbnail?: { url?: string }
      author?: { name?: string; url?: string; icon_url?: string }
      fields?: { name: string; value: string; inline: boolean }[]
    }[]
    attachments?: { Url: { url: string; filename?: string; description?: string } } | {
      Base64: { data: string; filename: string; description?: string }
    }[]
    components?:
      | number
      | string
      | boolean
      | Array<JsonValue>
      | { [key in string]?: JsonValue }
      | null[]
    allowed_mentions?: {
      parse?: string[]
      users?: string[]
      roles?: string[]
      replied_user?: boolean
    }
    flags?: bigint
    thread_name?: string
  }

  type RawEditWebhook = {
    webhook_id: string
    token?: string
    payload: number | string | boolean | Array<JsonValue> | { [key in string]?: JsonValue } | null
    reason?: string
  }

  type RawDeleteWebhook = { webhook_id: string; token?: string; reason?: string }

  const flora: typeof import('./src/index')
}

export {}
