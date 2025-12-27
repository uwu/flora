# Runtime & SDK API

Reference documentation fo flora.

## Runtime API (global, always available)

- `on(event: string, handler: (ctx) => void | Promise<void>)`: register a handler for a Discord event. Multiple handlers per event are allowed.
- `ctx.msg`: the raw event payload passed from the Rust runtime.
- `ctx.reply(message: string | MessageReplyOptions): Promise<void>`: send a message back to the same channel. Strings reply inline to the triggering message when an ID is present; pass an object to use embeds, attachments, allowed mentions, or to opt out of replying.
- `console.log(...args)`: forwarded to Rust tracing (`flora:js`) for structured logs. This is temporary.

### Event names and payload shapes

Events mirror Serenity events bridged in `src/discord_handler.rs`.

```ts
type MessageAuthor = {
  id: string;
  username: string;
  discriminator?: number | null;
  bot: boolean;
};

type MessagePayload = {
  id: string;
  channel_id: string;
  guild_id?: string | null;
  content: string;
  author: MessageAuthor;
};

type MessageUpdatePayload = {
  id: string;
  channel_id: string;
  guild_id?: string | null;
  content?: string | null;
  author?: MessageAuthor | null;
  edited_timestamp?: string | null;
  old?: MessagePayload | null;
  new?: MessagePayload | null;
};

// Events you can subscribe to
on("ready", (ctx) => {
  /* ctx.msg: { user, guild_ids } */
});
on("messageCreate", (ctx: { msg: MessagePayload; reply: Function }) => {
  /* … */
});
on("messageUpdate", (ctx: { msg: MessageUpdatePayload; reply: Function }) => {
  /* … */
});
on("messageDelete", (ctx) => {
  /* ctx.msg: { id, channel_id, guild_id? } */
});
on("messageDeleteBulk", (ctx) => {
  /* ctx.msg: { ids: string[], channel_id, guild_id? } */
});
```

Notes:

- Handlers run inside a single-threaded V8 isolate per guild; avoid blocking work.
- `reply` accepts either a string or a `MessageReplyOptions` object (see below). Strings are replied inline; set `replyTo: null` in the options to send without referencing the original message.
- If no guild-specific isolate exists, events fall back to the default runtime.

`MessageReplyOptions` shape:

```ts
type MessageReplyOptions = {
  content?: string;
  embeds?: Array<{
    title?: string;
    description?: string;
    url?: string;
    color?: number; // hex RGB, e.g. 0x5865f2
    footer?: { text: string; iconUrl?: string };
    image?: { url: string };
    thumbnail?: { url: string };
    author?: { name?: string; url?: string; iconUrl?: string };
    fields?: Array<{ name: string; value: string; inline?: boolean }>;
  }>;
  attachments?: Array<
    | { url: string; filename?: string; description?: string }
    | { data: string; filename: string; description?: string }
  >; // data expects base64
  tts?: boolean;
  allowedMentions?: {
    parse?: ("everyone" | "roles" | "users")[];
    users?: string[];
    roles?: string[];
    repliedUser?: boolean;
  };
  replyTo?: string | null; // override or disable auto-reply
  ephemeral?: boolean; // only for interaction replies
};
```

## SDK API (imported from `dist/sdk-bundle.js`)

The SDK builds on the runtime helpers to simplify prefix-style commands.

```ts
/// defineCommand, createBot are globally available, see example/ dir
const ping = defineCommand({
  name: "ping",
  description: "Respond with pong",
  run: async (ctx) => {
    await ctx.reply(`pong (${ctx.args.join(" ") || "no args"})`);

    // Rich reply with embeds, attachment, and mention controls
    await ctx.reply({
      content: "Here is your data",
      embeds: [
        {
          title: "Daily Status",
          description: "All systems nominal",
          color: 0x00ff99,
          fields: [
            { name: "Shard", value: "east-1" },
            { name: "Latency", value: "42ms", inline: true },
          ],
        },
      ],
      attachments: [
        { url: "https://example.com/report.csv", filename: "report.csv" },
      ],
      allowedMentions: { parse: ["users"], repliedUser: false },
    });
  },
});

createBot({
  prefix: "!", // optional; defaults to "!"
  commands: [ping], // or use prefixCommands for legacy naming
});
```

### Exports

- `defineCommand(command: { name: string; description?: string; run(ctx): void | Promise<void> })`: returns the command unchanged; use it for type safety and clarity.
- `createBot(options)`: wires message handlers for prefix commands.
- `options.prefix?: string` — command prefix (default `"!"`).
- `options.commands?: Command[]` — commands to register (preferred).
- `options.prefixCommands?: Command[]` — alias of `commands` for compatibility.
- `options.slashCommands?: SlashCommand[]` — handlers invoked for `interactionCreate` slash events.
- Types re-exported for consumers: `MessageAuthor`, `MessagePayload`, `MessageContext`, `MessageUpdatePayload`, `MessageUpdateContext`, `MessageDeletePayload`, `MessageDeleteContext`, `MessageDeleteBulkPayload`, `MessageDeleteBulkContext`, `Command`.
- Types re-exported for slash commands: `SlashCommand`, `SlashCommandOption`, `InteractionContext`, `InteractionPayload`.

Slash commands

```ts
const slashPing = defineSlashCommand({
  name: "ping",
  description: "Replies with pong",
  run: async (ctx) => {
    await ctx.reply({ content: "pong", ephemeral: true });
  },
});

const slashEcho = defineSlashCommand({
  name: "echo",
  description: "Echo back your input",
  options: [
    {
      name: "text",
      description: "What should I repeat?",
      type: "string",
      required: true,
    },
  ],
  async run(ctx) {
    const content = ctx.msg?.data?.options?.[0]?.value ?? "(nothing)";
    await ctx.reply({ content });
  },
});

createBot({ slashCommands: [slashPing, slashEcho] });
```

- `ctx.msg` matches the `InteractionPayload` shape (ids, token, user, locale, command name, raw `data`).
- `ctx.reply` routes through interaction responses; pass `ephemeral: true` for private replies.
- Slash command options support types: `string` (default), `integer`, `number`, `boolean`; set `required: true` as needed.

### How command dispatch works

- The SDK registers an internal `on("messageCreate")` handler.
- Incoming messages are ignored if authored by bots or if the content does not start with the configured prefix.
- The first token after the prefix is matched against `command.name`; remaining tokens are passed as `ctx.args`.
- `ctx.reply` routes through the runtime `op_send_message` to Discord with a message reference when possible.

## KV Store API

Persistent key-value storage for bot data, scoped per guild.

### Creating a store

KV stores must be created via the backend API or CLI before they can be used in bot scripts. Each store is isolated to a specific guild.

### Basic usage

```ts
import { kv } from '@flora/sdk'

// Get a named store instance
const userStore = kv.store('users')

// Set a value (max 1MB)
await userStore.set('alice', JSON.stringify({ name: 'Alice', score: 100 }))

// Get a value
const data = await userStore.get('alice')
// Returns string or null if not found
if (data) {
  const user = JSON.parse(data)
  console.log(user.name) // "Alice"
}

// Delete a key
await userStore.delete('alice')

// List all keys
const keys = await userStore.listKeys()
console.log(keys) // ["alice", "bob", ...]

// List keys with prefix filter
const userKeys = await userStore.listKeys('user')
console.log(userKeys) // Only keys starting with "user"
```

### Store methods

- `kv.store(name: string): KvStore` — Get a named KV store instance
- `KvStore.get(key: string): Promise<string | null>` — Get value by key
- `KvStore.set(key: string, value: string): Promise<void>` — Set value (max 1MB)
- `KvStore.delete(key: string): Promise<void>` — Delete key
- `KvStore.listKeys(prefix?: string): Promise<string[]>` — List all keys (optionally filtered by prefix)

### Important notes

- **Store creation**: Use the backend API or CLI to create stores before using them
- **Value size limit**: 1MB per value (enforced by backend)
- **Guild isolation**: Each guild's KV data is stored separately
- **Persistence**: Data is persisted to disk and survives bot restarts
- **List performance**: `listKeys()` is not paginated. It may be slow for stores with millions of keys. Consider using a prefix filter for better performance.

## Development tips

- Type definitions live in `dist/types` for editor intellisense when consuming the bundled SDK.
- Rebuild the bundle after SDK edits: `bun run sdk/build.ts` (run from repo root).
- For custom scripts outside the SDK, rely on the runtime globals (`on`, `console.log`, `ctx.reply`) and keep the event payload shapes above handy.

## Deploy API payloads

Deployments are uploaded as a multi-file bundle. The request body is:

```json
{
  "entry": "src/main.ts",
  "files": [
    { "path": "src/main.ts", "contents": "..." },
    { "path": "src/utils/reply.ts", "contents": "..." }
  ]
}
```

Notes:

- Only relative module imports (`./` and `../`) are supported today.
- The server bundles and runs the result as `guild:<id>.bundle.js` with an inline source map.
