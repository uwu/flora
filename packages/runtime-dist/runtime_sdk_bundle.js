var flora = (function (exports) {
  Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' })
  //#region src/sdk/commands.ts
  function prefix(command) {
    return command
  }
  function slash(command) {
    return command
  }
  function getCreateBotState() {
    const state = globalThis.__floraCreateBotState
    if (state) return state
    const initialState = { initialized: false }
    globalThis.__floraCreateBotState = initialState
    return initialState
  }
  function createBot(options) {
    const state = getCreateBotState()
    if (state.initialized) {
      console.log('[flora/sdk] createBot called multiple times; skipping duplicate registration')
      return
    }
    state.initialized = true
    const prefix = options.prefix ?? '!'
    const commands = options.commands ?? options.prefixCommands ?? []
    const slashCommands = options.slashCommands ?? []
    on('messageCreate', async (ctx) => {
      if (!ctx.msg || !ctx.msg.content) return
      if (ctx.msg.author?.bot) return
      const content = ctx.msg.content.trim()
      if (!content.startsWith(prefix)) return
      const [commandName, ...args] = content.slice(prefix.length).trim().split(/\s+/)
      const command = commands.find((cmd) => cmd.name === commandName)
      if (!command) return
      await command.run({
        ...ctx,
        args
      })
    })
    on('interactionCreate', async (ctx) => {
      if (!ctx.msg) return
      const command = slashCommands.find((cmd) => cmd.name === ctx.msg.commandName)
      if (!command) return
      if (command.subcommands && command.subcommands.length > 0)
        await handleSubcommand(ctx, command)
      else if (command.run) {
        const rawData = ctx.msg.data
        const options = flattenInteractionOptions(rawData?.options || [])
        await command.run({
          ...ctx,
          options
        })
      }
    })
    if (slashCommands.length && typeof registerSlashCommands === 'function') {
      const flattenedCommands = flattenCommands(slashCommands)
      registerSlashCommands(flattenedCommands)
    }
  }
  function flattenCommands(commands) {
    const subcommands = globalThis.__floraSubcommands
    globalThis.__floraSubcommands = subcommands || {}
    return commands.map((cmd) => {
      if (cmd.subcommands && cmd.subcommands.length > 0) {
        const submap = {}
        cmd.subcommands.forEach((sub) => {
          submap[sub.name] = sub.run
        })
        globalThis.__floraSubcommands[cmd.name] = submap
        return {
          name: cmd.name,
          description: cmd.description,
          options: cmd.subcommands.map((sub) => ({
            name: sub.name,
            description: sub.description,
            type: 'subcommand',
            options: sub.options
          }))
        }
      }
      return {
        name: cmd.name,
        description: cmd.description,
        options: cmd.options
      }
    })
  }
  async function handleSubcommand(ctx, command) {
    const rawData = ctx.msg.data
    if (!rawData?.options || !Array.isArray(rawData.options)) return
    const firstOption = rawData.options[0]
    if (!firstOption) return
    const subcommandName = firstOption.name
    const subcommandMap = globalThis.__floraSubcommands?.[command.name]
    if (!subcommandMap) return
    const subcommandHandler = subcommandMap[subcommandName]
    if (!subcommandHandler) return
    const flatOptions = flattenInteractionOptions(firstOption.options || [])
    await subcommandHandler({
      ...ctx,
      options: flatOptions
    })
  }
  function flattenInteractionOptions(options) {
    const result = {}
    for (const opt of options)
      if (opt.type === 1 || opt.type === 2)
        Object.assign(result, flattenInteractionOptions(opt.options || []))
      else result[opt.name] = opt.value
    return result
  }
  //#endregion
  //#region src/generated.ts
  const ButtonStyle = {
    Primary: 1,
    Secondary: 2,
    Success: 3,
    Danger: 4
  }
  const ComponentType = {
    ActionRow: 1,
    Button: 2,
    StringSelect: 3,
    InputText: 4,
    UserSelect: 5,
    RoleSelect: 6,
    MentionableSelect: 7,
    ChannelSelect: 8,
    Section: 9,
    TextDisplay: 10,
    Thumbnail: 11,
    MediaGallery: 12,
    File: 13,
    Separator: 14,
    Container: 17,
    Label: 18,
    FileUpload: 19
  }
  const InputTextStyle = {
    Short: 1,
    Paragraph: 2
  }
  const MessageFlags = {
    CROSSPOSTED: 1,
    IS_CROSSPOST: 2,
    SUPPRESS_EMBEDS: 4,
    SOURCE_MESSAGE_DELETED: 8,
    URGENT: 16,
    HAS_THREAD: 32,
    EPHEMERAL: 64,
    LOADING: 128,
    FAILED_TO_MENTION_SOME_ROLES_IN_THREAD: 256,
    SUPPRESS_NOTIFICATIONS: 4096,
    IS_VOICE_MESSAGE: 8192,
    IS_COMPONENTS_V2: 32768
  }
  //#endregion
  //#region src/sdk/components.ts
  const isBuilder = (value) => typeof value?.toJSON === 'function'
  const resolveComponent = (value) => (isBuilder(value) ? value.toJSON() : value)
  var ActionRowBuilder = class {
    #components = []
    addComponents(...components) {
      this.#components.push(...components)
      return this
    }
    setComponents(components) {
      this.#components = [...components]
      return this
    }
    toJSON() {
      return {
        type: ComponentType.ActionRow,
        components: this.#components.map(resolveComponent)
      }
    }
  }
  var ButtonBuilder = class {
    #data = { type: ComponentType.Button }
    setStyle(style) {
      this.#data.style = style
      return this
    }
    setCustomId(customId) {
      this.#data.custom_id = customId
      return this
    }
    setUrl(url) {
      this.#data.style = 5
      this.#data.url = url
      return this
    }
    setSkuId(skuId) {
      this.#data.style = 6
      this.#data.sku_id = skuId
      return this
    }
    setLabel(label) {
      this.#data.label = label
      return this
    }
    setEmoji(emoji) {
      this.#data.emoji = emoji
      return this
    }
    setDisabled(disabled = true) {
      this.#data.disabled = disabled
      return this
    }
    toJSON() {
      return { ...this.#data }
    }
  }
  var SelectMenuBuilderBase = class {
    data
    constructor(type, customId) {
      this.data = {
        type,
        custom_id: customId
      }
    }
    setCustomId(customId) {
      this.data.custom_id = customId
      return this
    }
    setPlaceholder(placeholder) {
      this.data.placeholder = placeholder
      return this
    }
    setMinValues(min) {
      this.data.min_values = min
      return this
    }
    setMaxValues(max) {
      this.data.max_values = max
      return this
    }
    setRequired(required = true) {
      this.data.required = required
      return this
    }
    setDisabled(disabled = true) {
      this.data.disabled = disabled
      return this
    }
    setChannelTypes(types) {
      this.data.channel_types = [...types]
      return this
    }
    setDefaultValues(values) {
      this.data.default_values = values.map((value) => ({ ...value }))
      return this
    }
    setDefaultUsers(ids) {
      this.data.default_users = [...ids]
      return this
    }
    setDefaultRoles(ids) {
      this.data.default_roles = [...ids]
      return this
    }
    setDefaultChannels(ids) {
      this.data.default_channels = [...ids]
      return this
    }
    addDefaultUser(id) {
      const current = this.data.default_users ?? []
      this.data.default_users = [...current, id]
      return this
    }
    addDefaultRole(id) {
      const current = this.data.default_roles ?? []
      this.data.default_roles = [...current, id]
      return this
    }
    addDefaultChannel(id) {
      const current = this.data.default_channels ?? []
      this.data.default_channels = [...current, id]
      return this
    }
    addDefaultValue(id, type) {
      const current = this.data.default_values ?? []
      this.data.default_values = [
        ...current,
        {
          id,
          type
        }
      ]
      return this
    }
    toJSON() {
      return { ...this.data }
    }
  }
  var StringSelectMenuBuilder = class extends SelectMenuBuilderBase {
    constructor(customId) {
      super(ComponentType.StringSelect, customId)
    }
    setOptions(options) {
      this.data.options = options.map((option) => ({ ...option }))
      return this
    }
    addOptions(...options) {
      const current = this.data.options ?? []
      this.data.options = [...current, ...options.map((option) => ({ ...option }))]
      return this
    }
  }
  var UserSelectMenuBuilder = class extends SelectMenuBuilderBase {
    constructor(customId) {
      super(ComponentType.UserSelect, customId)
    }
  }
  var RoleSelectMenuBuilder = class extends SelectMenuBuilderBase {
    constructor(customId) {
      super(ComponentType.RoleSelect, customId)
    }
  }
  var MentionableSelectMenuBuilder = class extends SelectMenuBuilderBase {
    constructor(customId) {
      super(ComponentType.MentionableSelect, customId)
    }
  }
  var ChannelSelectMenuBuilder = class extends SelectMenuBuilderBase {
    constructor(customId) {
      super(ComponentType.ChannelSelect, customId)
    }
  }
  var InputTextBuilder = class {
    #data
    constructor(customId) {
      this.#data = {
        type: ComponentType.InputText,
        custom_id: customId
      }
    }
    setCustomId(customId) {
      this.#data.custom_id = customId
      return this
    }
    setStyle(style) {
      this.#data.style = style
      return this
    }
    setMinLength(min) {
      this.#data.min_length = min
      return this
    }
    setMaxLength(max) {
      this.#data.max_length = max
      return this
    }
    setRequired(required = true) {
      this.#data.required = required
      return this
    }
    setValue(value) {
      this.#data.value = value
      return this
    }
    setPlaceholder(placeholder) {
      this.#data.placeholder = placeholder
      return this
    }
    toJSON() {
      return { ...this.#data }
    }
  }
  var TextDisplayBuilder = class {
    #data
    constructor(content) {
      this.#data = {
        type: ComponentType.TextDisplay,
        content
      }
    }
    setContent(content) {
      this.#data.content = content
      return this
    }
    toJSON() {
      return { ...this.#data }
    }
  }
  var ThumbnailBuilder = class {
    #data
    constructor(url) {
      this.#data = {
        type: ComponentType.Thumbnail,
        media: { url }
      }
    }
    setUrl(url) {
      this.#data.media = { url }
      return this
    }
    setDescription(description) {
      this.#data.description = description
      return this
    }
    setSpoiler(spoiler = true) {
      this.#data.spoiler = spoiler
      return this
    }
    toJSON() {
      return { ...this.#data }
    }
  }
  var SectionBuilder = class {
    #components = []
    #accessory
    addComponents(...components) {
      this.#components.push(...components)
      return this
    }
    setComponents(components) {
      this.#components = [...components]
      return this
    }
    setAccessory(accessory) {
      this.#accessory = accessory
      return this
    }
    toJSON() {
      return {
        type: ComponentType.Section,
        components: this.#components.map(resolveComponent),
        accessory: this.#accessory ? resolveComponent(this.#accessory) : null
      }
    }
  }
  var MediaGalleryBuilder = class {
    #items = []
    addItem(url, options) {
      const item = {
        media: { url },
        description: options?.description,
        spoiler: options?.spoiler
      }
      this.#items.push(item)
      return this
    }
    setItems(items) {
      this.#items = items.map((item) => ({
        ...item,
        media: { ...item.media }
      }))
      return this
    }
    toJSON() {
      return {
        type: ComponentType.MediaGallery,
        items: this.#items.map((item) => ({
          ...item,
          media: { ...item.media }
        }))
      }
    }
  }
  var FileBuilder = class {
    #data
    constructor(url) {
      this.#data = {
        type: ComponentType.File,
        file: { url }
      }
    }
    setUrl(url) {
      this.#data.file = { url }
      return this
    }
    setSpoiler(spoiler = true) {
      this.#data.spoiler = spoiler
      return this
    }
    toJSON() {
      return { ...this.#data }
    }
  }
  var SeparatorBuilder = class {
    #data
    constructor(divider = true) {
      this.#data = {
        type: ComponentType.Separator,
        divider
      }
    }
    setDivider(divider = true) {
      this.#data.divider = divider
      return this
    }
    setSpacing(spacing) {
      this.#data.spacing = spacing
      return this
    }
    toJSON() {
      return { ...this.#data }
    }
  }
  var ContainerBuilder = class {
    #components = []
    #accentColor
    #spoiler
    addComponents(...components) {
      this.#components.push(...components)
      return this
    }
    setComponents(components) {
      this.#components = [...components]
      return this
    }
    setAccentColor(color) {
      this.#accentColor = color
      return this
    }
    setSpoiler(spoiler = true) {
      this.#spoiler = spoiler
      return this
    }
    toJSON() {
      const data = {
        type: ComponentType.Container,
        components: this.#components.map(resolveComponent)
      }
      if (this.#accentColor !== void 0) data.accent_color = this.#accentColor
      if (this.#spoiler !== void 0) data.spoiler = this.#spoiler
      return data
    }
  }
  var LabelBuilder = class {
    #label
    #description
    #component
    constructor(label) {
      this.#label = label
    }
    setLabel(label) {
      this.#label = label
      return this
    }
    setDescription(description) {
      this.#description = description
      return this
    }
    setComponent(component) {
      this.#component = component
      return this
    }
    toJSON() {
      const data = {
        type: ComponentType.Label,
        label: this.#label,
        component: this.#component ? resolveComponent(this.#component) : null
      }
      if (this.#description !== void 0) data.description = this.#description
      return data
    }
  }
  var FileUploadBuilder = class {
    #data
    constructor(customId) {
      this.#data = {
        type: ComponentType.FileUpload,
        custom_id: customId
      }
    }
    setCustomId(customId) {
      this.#data.custom_id = customId
      return this
    }
    setMinValues(min) {
      this.#data.min_values = min
      return this
    }
    setMaxValues(max) {
      this.#data.max_values = max
      return this
    }
    setRequired(required = true) {
      this.#data.required = required
      return this
    }
    toJSON() {
      return { ...this.#data }
    }
  }
  const actionRow = () => new ActionRowBuilder()
  const button = () => new ButtonBuilder()
  const stringSelect = (customId) => new StringSelectMenuBuilder(customId)
  const userSelect = (customId) => new UserSelectMenuBuilder(customId)
  const roleSelect = (customId) => new RoleSelectMenuBuilder(customId)
  const mentionableSelect = (customId) => new MentionableSelectMenuBuilder(customId)
  const channelSelect = (customId) => new ChannelSelectMenuBuilder(customId)
  const inputText = (customId) => new InputTextBuilder(customId)
  const textDisplay = (content) => new TextDisplayBuilder(content)
  const thumbnail = (url) => new ThumbnailBuilder(url)
  const section = () => new SectionBuilder()
  const mediaGallery = () => new MediaGalleryBuilder()
  const file = (url) => new FileBuilder(url)
  const separator = (divider = true) => new SeparatorBuilder(divider)
  const container = () => new ContainerBuilder()
  const label = (labelText) => new LabelBuilder(labelText)
  const fileUpload = (customId) => new FileUploadBuilder(customId)
  const ButtonStyles = ButtonStyle
  const InputTextStyles = InputTextStyle
  //#endregion
  //#region src/sdk/embed.ts
  var EmbedBuilder = class {
    #embed
    constructor(initial = {}) {
      this.#embed = { ...initial }
    }
    setTitle(title) {
      this.#embed.title = title
      return this
    }
    setDescription(description) {
      this.#embed.description = description
      return this
    }
    setUrl(url) {
      this.#embed.url = url
      return this
    }
    setColor(color) {
      this.#embed.color = color
      return this
    }
    setTimestamp(timestamp) {
      this.#embed.timestamp = timestamp
      return this
    }
    setFooter(text, iconUrl) {
      this.#embed.footer = {
        text,
        iconUrl
      }
      return this
    }
    setImage(url) {
      this.#embed.image = { url }
      return this
    }
    setThumbnail(url) {
      this.#embed.thumbnail = { url }
      return this
    }
    setAuthor(name, options) {
      this.#embed.author = {
        name,
        ...options
      }
      return this
    }
    addField(name, value, inline = false) {
      const field = {
        name,
        value,
        inline
      }
      this.#embed.fields = [...(this.#embed.fields ?? []), field]
      return this
    }
    addFields(fields) {
      this.#embed.fields = [...(this.#embed.fields ?? []), ...fields]
      return this
    }
    setFields(fields) {
      this.#embed.fields = [...fields]
      return this
    }
    toJSON() {
      return { ...this.#embed }
    }
  }
  function embed(initial) {
    return new EmbedBuilder(initial)
  }
  //#endregion
  //#region src/sdk/helpers.ts
  function hasRole(ctx, roleId) {
    return ctx.msg.member?.roles?.includes(roleId) ?? false
  }
  function getSubcommand(ctx) {
    const rawData = ctx.msg.data
    if (!rawData?.options || !Array.isArray(rawData.options)) return void 0
    return rawData.options[0]?.name
  }
  function getSubcommandGroup(ctx) {
    const rawData = ctx.msg.data
    if (!rawData?.options || !Array.isArray(rawData.options)) return void 0
    const firstOption = rawData.options[0]
    if (!firstOption) return void 0
    if (firstOption.type === 2) return firstOption.name
  }
  //#endregion
  //#region src/sdk/kv.ts
  var KvStore = class {
    #storeName
    constructor(storeName) {
      this.#storeName = storeName
    }
    /**
     * Get a value from the store.
     *
     * @param key - The key to retrieve
     * @returns The value, or null if not found
     */
    async get(key) {
      return await Deno.core.ops.op_kv_get(this.#storeName, key)
    }
    /**
     * Get a value from the store along with its metadata.
     *
     * @param key - The key to retrieve
     * @returns Object with value and optional metadata
     */
    async getWithMetadata(key) {
      const result = await Deno.core.ops.op_kv_get_with_metadata(this.#storeName, key)
      if (result === null) return { value: null }
      const [value, metadata] = result
      return {
        value,
        metadata: metadata?.metadata
      }
    }
    /**
     * Set a value in the store.
     *
     * The value size is limited to 1MB.
     *
     * @param key - The key to set
     * @param value - The value to store (max 1MB)
     * @param options - Optional expiration (Unix timestamp) and metadata
     */
    async set(key, value, options) {
      await Deno.core.ops.op_kv_set(this.#storeName, key, value, {
        expiration: options?.expiration ?? void 0,
        metadata: options?.metadata ?? void 0
      })
    }
    /**
     * Update just the metadata for a key without changing the value.
     *
     * @param key - The key to update
     * @param metadata - The metadata to set, or null to remove metadata
     */
    async updateMetadata(key, metadata) {
      await Deno.core.ops.op_kv_update_metadata(this.#storeName, key, metadata ?? void 0)
    }
    /**
     * Delete a key from the store.
     *
     * @param key - The key to delete
     */
    async delete(key) {
      await Deno.core.ops.op_kv_delete(this.#storeName, key)
    }
    /**
     * List all keys in the store with cursor-based pagination.
     *
     * @param options - Optional prefix filter, limit (default 100, max 1000), and cursor for pagination
     * @returns Paginated result with keys, list_complete flag, and cursor for next page
     */
    async list(options) {
      return await Deno.core.ops.op_kv_list_keys(
        {
          prefix: options?.prefix ?? void 0,
          limit: options?.limit ?? void 0,
          cursor: options?.cursor ?? void 0
        },
        this.#storeName
      )
    }
  }
  function store(name) {
    return new KvStore(name)
  }
  const kv = { store }
  //#endregion
  //#region src/sdk/rest.ts
  const ops = Deno.core.ops
  /**
   * Lightweight REST bindings over core ops.
   * Errors include a `code` field (e.g. DISCORD_RATE_LIMITED).
   */
  const rest = {
    sendMessage: (args) => ops.op_send_message(args),
    editMessage: (args) => ops.op_edit_message(args),
    deleteMessage: (args) => ops.op_delete_message(args),
    bulkDeleteMessages: (args) => ops.op_bulk_delete_messages(args),
    pinMessage: (args) => ops.op_pin_message(args),
    unpinMessage: (args) => ops.op_unpin_message(args),
    crosspostMessage: (args) => ops.op_crosspost_message(args),
    fetchMessage: (args) => ops.op_fetch_message(args),
    fetchMessages: (args) => ops.op_fetch_messages(args),
    addReaction: (args) => ops.op_add_reaction(args),
    removeReaction: (args) => ops.op_remove_reaction(args),
    clearReactions: (args) => ops.op_clear_reactions(args),
    sendInteractionResponse: (args) => ops.op_send_interaction_response(args),
    deferInteractionResponse: (args) => ops.op_defer_interaction_response(args),
    updateInteractionResponse: (args) => ops.op_update_interaction_response(args),
    editOriginalInteractionResponse: (args) => ops.op_edit_original_interaction_response(args),
    deleteOriginalInteractionResponse: (args) => ops.op_delete_original_interaction_response(args),
    createFollowupMessage: (args) => ops.op_create_followup_message(args),
    editFollowupMessage: (args) => ops.op_edit_followup_message(args),
    deleteFollowupMessage: (args) => ops.op_delete_followup_message(args),
    upsertGuildCommands: (args) => ops.op_upsert_guild_commands(args),
    createGuildCommand: (args) => ops.op_create_guild_command(args),
    editGuildCommand: (args) => ops.op_edit_guild_command(args),
    deleteGuildCommand: (args) => ops.op_delete_guild_command(args),
    getGuildCommands: (args) => ops.op_get_guild_commands(args),
    getGuildCommand: (args) => ops.op_get_guild_command(args),
    editGuildCommandPermissions: (args) => ops.op_edit_guild_command_permissions(args),
    getGuildCommandsPermissions: (args) => ops.op_get_guild_commands_permissions(args),
    getGuildCommandPermissions: (args) => ops.op_get_guild_command_permissions(args),
    kickMember: (args) => ops.op_kick_member(args),
    banMember: (args) => ops.op_ban_member(args),
    unbanMember: (args) => ops.op_unban_member(args),
    addMemberRole: (args) => ops.op_add_member_role(args),
    removeMemberRole: (args) => ops.op_remove_member_role(args),
    editMember: (args) => ops.op_edit_member(args),
    editCurrentMember: (args) => ops.op_edit_current_member(args),
    createChannel: (args) => ops.op_create_channel(args),
    editChannel: (args) => ops.op_edit_channel(args),
    deleteChannel: (args) => ops.op_delete_channel(args),
    createThread: (args) => ops.op_create_thread(args),
    createThreadFromMessage: (args) => ops.op_create_thread_from_message(args),
    joinThread: (args) => ops.op_join_thread(args),
    leaveThread: (args) => ops.op_leave_thread(args),
    addThreadMember: (args) => ops.op_add_thread_member(args),
    removeThreadMember: (args) => ops.op_remove_thread_member(args),
    executeWebhook: (args) => ops.op_execute_webhook(args),
    editWebhook: (args) => ops.op_edit_webhook(args),
    deleteWebhook: (args) => ops.op_delete_webhook(args)
  }
  //#endregion
  exports.ActionRowBuilder = ActionRowBuilder
  exports.ButtonBuilder = ButtonBuilder
  exports.ButtonStyle = ButtonStyle
  exports.ButtonStyles = ButtonStyles
  exports.ChannelSelectMenuBuilder = ChannelSelectMenuBuilder
  exports.ComponentType = ComponentType
  exports.ContainerBuilder = ContainerBuilder
  exports.EmbedBuilder = EmbedBuilder
  exports.FileBuilder = FileBuilder
  exports.FileUploadBuilder = FileUploadBuilder
  exports.InputTextBuilder = InputTextBuilder
  exports.InputTextStyle = InputTextStyle
  exports.InputTextStyles = InputTextStyles
  exports.KvStore = KvStore
  exports.LabelBuilder = LabelBuilder
  exports.MediaGalleryBuilder = MediaGalleryBuilder
  exports.MentionableSelectMenuBuilder = MentionableSelectMenuBuilder
  exports.MessageFlags = MessageFlags
  exports.RoleSelectMenuBuilder = RoleSelectMenuBuilder
  exports.SectionBuilder = SectionBuilder
  exports.SelectMenuBuilderBase = SelectMenuBuilderBase
  exports.SeparatorBuilder = SeparatorBuilder
  exports.StringSelectMenuBuilder = StringSelectMenuBuilder
  exports.TextDisplayBuilder = TextDisplayBuilder
  exports.ThumbnailBuilder = ThumbnailBuilder
  exports.UserSelectMenuBuilder = UserSelectMenuBuilder
  exports.actionRow = actionRow
  exports.button = button
  exports.channelSelect = channelSelect
  exports.container = container
  exports.createBot = createBot
  exports.embed = embed
  exports.file = file
  exports.fileUpload = fileUpload
  exports.flattenCommands = flattenCommands
  exports.flattenInteractionOptions = flattenInteractionOptions
  exports.getSubcommand = getSubcommand
  exports.getSubcommandGroup = getSubcommandGroup
  exports.handleSubcommand = handleSubcommand
  exports.hasRole = hasRole
  exports.inputText = inputText
  exports.kv = kv
  exports.label = label
  exports.mediaGallery = mediaGallery
  exports.mentionableSelect = mentionableSelect
  exports.prefix = prefix
  exports.rest = rest
  exports.roleSelect = roleSelect
  exports.section = section
  exports.separator = separator
  exports.slash = slash
  exports.store = store
  exports.stringSelect = stringSelect
  exports.textDisplay = textDisplay
  exports.thumbnail = thumbnail
  exports.userSelect = userSelect
  return exports
})({})
;(function (global) {
  if (!global.flora) return
  global.actionRow = global.flora.actionRow
  global.ActionRowBuilder = global.flora.ActionRowBuilder
  global.button = global.flora.button
  global.ButtonBuilder = global.flora.ButtonBuilder
  global.ButtonStyle = global.flora.ButtonStyle
  global.ButtonStyles = global.flora.ButtonStyles
  global.channelSelect = global.flora.channelSelect
  global.ChannelSelectMenuBuilder = global.flora.ChannelSelectMenuBuilder
  global.ComponentType = global.flora.ComponentType
  global.container = global.flora.container
  global.ContainerBuilder = global.flora.ContainerBuilder
  global.createBot = global.flora.createBot
  global.embed = global.flora.embed
  global.EmbedBuilder = global.flora.EmbedBuilder
  global.file = global.flora.file
  global.FileBuilder = global.flora.FileBuilder
  global.fileUpload = global.flora.fileUpload
  global.FileUploadBuilder = global.flora.FileUploadBuilder
  global.flattenCommands = global.flora.flattenCommands
  global.flattenInteractionOptions = global.flora.flattenInteractionOptions
  global.getSubcommand = global.flora.getSubcommand
  global.getSubcommandGroup = global.flora.getSubcommandGroup
  global.handleSubcommand = global.flora.handleSubcommand
  global.hasRole = global.flora.hasRole
  global.inputText = global.flora.inputText
  global.InputTextBuilder = global.flora.InputTextBuilder
  global.InputTextStyle = global.flora.InputTextStyle
  global.InputTextStyles = global.flora.InputTextStyles
  global.kv = global.flora.kv
  global.KvStore = global.flora.KvStore
  global.label = global.flora.label
  global.LabelBuilder = global.flora.LabelBuilder
  global.mediaGallery = global.flora.mediaGallery
  global.MediaGalleryBuilder = global.flora.MediaGalleryBuilder
  global.mentionableSelect = global.flora.mentionableSelect
  global.MentionableSelectMenuBuilder = global.flora.MentionableSelectMenuBuilder
  global.MessageFlags = global.flora.MessageFlags
  global.prefix = global.flora.prefix
  global.rest = global.flora.rest
  global.roleSelect = global.flora.roleSelect
  global.RoleSelectMenuBuilder = global.flora.RoleSelectMenuBuilder
  global.section = global.flora.section
  global.SectionBuilder = global.flora.SectionBuilder
  global.SelectMenuBuilderBase = global.flora.SelectMenuBuilderBase
  global.separator = global.flora.separator
  global.SeparatorBuilder = global.flora.SeparatorBuilder
  global.slash = global.flora.slash
  global.store = global.flora.store
  global.stringSelect = global.flora.stringSelect
  global.StringSelectMenuBuilder = global.flora.StringSelectMenuBuilder
  global.textDisplay = global.flora.textDisplay
  global.TextDisplayBuilder = global.flora.TextDisplayBuilder
  global.thumbnail = global.flora.thumbnail
  global.ThumbnailBuilder = global.flora.ThumbnailBuilder
  global.userSelect = global.flora.userSelect
  global.UserSelectMenuBuilder = global.flora.UserSelectMenuBuilder
})(globalThis)
