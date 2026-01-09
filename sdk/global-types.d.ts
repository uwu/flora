// Auto-generated global types for Flora SDK
// Do not edit manually - regenerate with `bun run build`

declare global {

  // Runtime globals (from runtime_prelude.js)
  interface FloraEventMap {
    ready: BaseContext<EventReady>
    messageCreate: MessageContext
    messageUpdate: MessageUpdateContext
    messageDelete: MessageDeleteContext
    messageDeleteBulk: MessageDeleteBulkContext
    interactionCreate: InteractionContext
  }

  function on<E extends keyof FloraEventMap>(event: E, handler: (ctx: FloraEventMap[E]) => void | Promise<void>): void
  function registerSlashCommands(commands: FlattenedSlashCommand[]): void

  const __floraHandlers: Record<string, Function[]>
  const __floraGuildId: string | undefined
  function __floraDispatch(event: string, payload: unknown): Promise<void>

  const flora: typeof import('./src/index')


  // SDK exports (functions, consts, classes, types)
  function prefix(command: { name: string; description?: string; run: (ctx: MessageContext & { args: string[]; }) => Promise<void> | void; }): { name: string; description?: string; run: (ctx: MessageContext & { args: string[]; }) => Promise<void> | void; };

  function slash(command: { name: string; description: string; options?: SlashCommandOption[]; subcommands?: SlashSubcommand[]; run?: (ctx: InteractionContext) => Promise<void> | void; }): { name: string; description: string; options?: SlashCommandOption[]; subcommands?: SlashSubcommand[]; run?: (ctx: InteractionContext) => Promise<void> | void; };

  function createBot(options: { prefix?: string; commands?: Command[]; prefixCommands?: Command[]; slashCommands?: SlashCommand[]; }): void;

  function flattenCommands(commands: Array<SlashCommand>): Array<FlattenedSlashCommand>;

  function handleSubcommand(ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions; }, command: { name: string; description: string; options?: SlashCommandOption[]; subcommands?: SlashSubcommand[]; run?: (ctx: InteractionContext) => Promise<void> | void; }): Promise<void>;

  function flattenInteractionOptions(options: Array<any>): { [x: string]: any; };

  type Command = { name: string; description?: string; run: (ctx: MessageContext & { args: string[]; }) => Promise<void> | void; };

  type SlashCommandOption = { name: string; description: string; type?: "string" | "integer" | "number" | "boolean" | "subcommand" | "subcommand_group"; required?: boolean; options?: SlashCommandOption[]; };

  type SlashSubcommand = { name: string; description: string; options?: SlashCommandOption[]; run: (ctx: InteractionContext) => Promise<void> | void; };

  type SlashCommand = { name: string; description: string; options?: SlashCommandOption[]; subcommands?: SlashSubcommand[]; run?: (ctx: InteractionContext) => Promise<void> | void; };

  type CreateOptions = { prefix?: string; commands?: Command[]; prefixCommands?: Command[]; slashCommands?: SlashCommand[]; };

  type FlattenedSlashCommand = { name: string; description: string; options?: SlashCommandOption[]; };

  type SubcommandMap = { [x: string]: Record<string, (ctx: InteractionContext) => Promise<void> | void>; };

  function embed(initial?: Embed | undefined): EmbedBuilder;

  class EmbedBuilder {
    setTitle(title: string): this;
    setDescription(description: string): this;
    setUrl(url: string): this;
    setColor(color: number): this;
    setTimestamp(timestamp: string): this;
    setFooter(text: string, iconUrl?: string | undefined): this;
    setImage(url: string): this;
    setThumbnail(url: string): this;
    setAuthor(name?: string | undefined, options?: { url?: string; iconUrl?: string; } | undefined): this;
    addField(name: string, value: string, inline?: boolean): this;
    addFields(fields: Array<EmbedField>): this;
    setFields(fields: Array<EmbedField>): this;
    toJSON(): { title?: string; description?: string; url?: string; color?: number; timestamp?: string; footer?: RawEmbedFooter; image?: RawEmbedMedia; thumbnail?: RawEmbedMedia; author?: RawEmbedAuthor; fields?: Array<EmbedField>; };
  }

  function hasRole(ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions; }, roleId: string): boolean;

  function getSubcommand(ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions; }): string | undefined;

  function getSubcommandGroup(ctx: BaseContext<EventInteractionCreate> & { options: SlashCommandOptions; }): string | undefined;

  function store(name: string): KvStore;

  type RawKvGetResult = { value: string | null; metadata?: Record<string, unknown>; };

  class KvStore {
    get(key: string): Promise<string | null>;
    getWithMetadata(key: string): Promise<RawKvGetResult>;
    set(key: string, value: string, options?: RawKvSetOptions | undefined): Promise<void>;
    updateMetadata(key: string, metadata: JsonValue | undefined): Promise<void>;
    delete(key: string): Promise<void>;
    list(options?: RawKvListKeysOptions | undefined): Promise<RawKvListKeysResult>;
  }

  const kv: { store: (name: string) => KvStore; };

  type Embed = { title?: string; description?: string; url?: string; color?: number; timestamp?: string; footer?: RawEmbedFooter; image?: RawEmbedMedia; thumbnail?: RawEmbedMedia; author?: RawEmbedAuthor; fields?: Array<RawEmbedField>; };

  type EmbedField = { name: string; value: string; inline: boolean; };

  type MessageReplyOptions = { content?: string; embeds?: RawEmbed[]; attachments?: RawAttachment[]; tts?: boolean; allowedMentions?: RawAllowedMentions; replyTo?: string | null; ephemeral?: boolean; flags?: number; };

  type MessageEditOptions = { content?: string; embeds?: RawEmbed[]; allowedMentions?: RawAllowedMentions; flags?: number; };

  type BaseContext<TPayload> = { msg: TPayload; reply: (content: string | MessageReplyOptions) => Promise<void>; edit: (content: string | MessageEditOptions) => Promise<void>; };

  type MessageContext = { msg: EventMessage; reply: (content: string | MessageReplyOptions) => Promise<void>; edit: (content: string | MessageEditOptions) => Promise<void>; };

  type MessageUpdateContext = { msg: EventMessageUpdate; reply: (content: string | MessageReplyOptions) => Promise<void>; edit: (content: string | MessageEditOptions) => Promise<void>; };

  type MessageDeleteContext = { msg: EventMessageDelete; reply: (content: string | MessageReplyOptions) => Promise<void>; edit: (content: string | MessageEditOptions) => Promise<void>; };

  type MessageDeleteBulkContext = { msg: EventMessageDeleteBulk; reply: (content: string | MessageReplyOptions) => Promise<void>; edit: (content: string | MessageEditOptions) => Promise<void>; };

  type SlashCommandOptions = { [x: string]: string | number | boolean | undefined; };

  type InteractionContext = BaseContext<EventInteractionCreate> & { options: SlashCommandOptions; };

  type JsonValue = string | number | boolean | Array<JsonValue> | { [x: string]: JsonValue | undefined; } | null;

  type EventInteractionCreate = { interactionId: string; interactionToken: string; applicationId: string; guildId?: string; channelId?: string; user: EventUser; member?: EventMember; commandName: string; data: JsonValue; locale?: string; guildLocale?: string; };

  type EventMember = { user: EventUser; nick?: string; avatar?: string; roles: Array<string>; joinedAt?: string; premiumSince?: string; deaf: boolean; mute: boolean; flags: number; pending: boolean; permissions?: string; communicationDisabledUntil?: string; };

  type EventMessage = { id: string; channelId: string; guildId?: string; content: string; author: EventUser; member?: EventMember; };

  type EventMessageDelete = { id: string; channelId: string; guildId?: string; };

  type EventMessageDeleteBulk = { ids: Array<string>; channelId: string; guildId?: string; };

  type EventMessageUpdate = { id: string; channelId: string; guildId?: string; content?: string; author?: EventUser; editedTimestamp?: string; old?: EventMessage; new?: EventMessage; };

  type EventReady = { user: EventUser; guildIds: Array<string>; };

  type EventUser = { id: string; username: string; discriminator?: number; bot: boolean; };

  type RawAllowedMentions = { parse?: Array<string>; users?: Array<string>; roles?: Array<string>; repliedUser?: boolean; };

  type RawAttachment = { url: string; filename: string | null; description: string | null; } | { data: string; filename: string; description: string | null; };

  type RawEditMessage = { channelId: string; messageId: string; content?: string; embeds?: Array<RawEmbed>; allowedMentions?: RawAllowedMentions; flags?: bigint; };

  type RawEmbed = { title?: string; description?: string; url?: string; color?: number; timestamp?: string; footer?: RawEmbedFooter; image?: RawEmbedMedia; thumbnail?: RawEmbedMedia; author?: RawEmbedAuthor; fields?: Array<RawEmbedField>; };

  type RawEmbedAuthor = { name?: string; url?: string; iconUrl?: string; };

  type RawEmbedField = { name: string; value: string; inline: boolean; };

  type RawEmbedFooter = { text?: string; iconUrl?: string; };

  type RawEmbedMedia = { url?: string; };

  type RawInteractionResponse = { interactionId: string; token: string; content?: string; embeds?: Array<RawEmbed>; attachments?: Array<RawAttachment>; tts?: boolean; allowedMentions?: RawAllowedMentions; ephemeral?: boolean; };

  type RawKvKeyInfo = { name: string; expiration?: bigint; metadata?: JsonValue; };

  type RawKvKeyMetadata = { expiration?: bigint; metadata?: JsonValue; };

  type RawKvListKeysOptions = { prefix?: string; limit?: bigint; cursor?: string; };

  type RawKvListKeysResult = { keys: Array<RawKvKeyInfo>; listComplete: boolean; cursor?: string; };

  type RawKvSetOptions = { expiration?: bigint; metadata?: JsonValue; };

  type RawSendMessage = { channelId: string; content?: string; embeds?: Array<RawEmbed>; attachments?: Array<RawAttachment>; tts?: boolean; allowedMentions?: RawAllowedMentions; flags?: bigint; messageId?: string; replyTo?: string; };

  type RawSlashCommand = { name: string; description?: string; options?: Array<RawSlashCommandOption>; };

  type RawSlashCommandOption = { name: string; description: string; type?: string; required?: boolean; options?: Array<RawSlashCommandOption>; };

  type RawUpsertGuildCommands = { guildId: string; commands: Array<RawSlashCommand>; };
}

export {}
