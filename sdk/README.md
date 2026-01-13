# Flora SDK

TypeScript SDK for building Discord bots with Flora.

## Overview

The SDK provides a type-safe API for building Discord bots. Types are auto-generated from the Rust runtime using `ts-rs`, ensuring they stay in sync with the backend.

## Quick Start

```ts
// All SDK functions are available globally - no imports needed!

const ping = defineCommand({
  name: 'ping',
  description: 'Respond with pong',
  run: async (ctx) => {
    await ctx.reply(`pong! args: ${ctx.args.join(' ') || 'none'}`)
  }
})

createBot({
  prefix: '!',
  commands: [ping]
})
```

## Runtime API (Always Available)

These globals are injected by the Flora runtime:

### Event Handlers

```ts
// Register event handlers
on('messageCreate', async (ctx) => {
  console.log(`Message from ${ctx.msg.author.username}: ${ctx.msg.content}`)
})

on('messageUpdate', async (ctx) => {
  console.log(`Message ${ctx.msg.id} was edited`)
})

on('messageDelete', async (ctx) => {
  console.log(`Message ${ctx.msg.id} was deleted`)
})

on('interactionCreate', async (ctx) => {
  console.log(`Slash command: ${ctx.msg.command_name}`)
})
```

### Context Methods

Every event handler receives a context object with:

- `ctx.msg` - The raw event payload
- `ctx.reply(message)` - Send a reply (string or options object)
- `ctx.edit(options)` - Edit the original message

## SDK API

### Prefix Commands

```ts
const greet = defineCommand({
  name: 'greet',
  description: 'Greet someone',
  run: async (ctx) => {
    const name = ctx.args[0] || 'world'
    await ctx.reply(`Hello, ${name}!`)
  }
})

createBot({
  prefix: '!', // Default: "!"
  commands: [greet]
})
```

### Slash Commands

```ts
const echo = defineSlashCommand({
  name: 'echo',
  description: 'Echo your input',
  options: [
    {
      name: 'text',
      description: 'Text to echo',
      type: 'string',
      required: true
    }
  ],
  run: async (ctx) => {
    const text = ctx.options.text as string
    await ctx.reply({ content: text, ephemeral: true })
  }
})

createBot({
  slashCommands: [echo]
})
```

### Slash Command Subcommands

```ts
const settings = defineSlashCommand({
  name: 'settings',
  description: 'Manage settings',
  subcommands: [
    {
      name: 'get',
      description: 'Get a setting',
      options: [{ name: 'key', description: 'Setting key', type: 'string', required: true }],
      run: async (ctx) => {
        const key = ctx.options.key as string
        await ctx.reply(`Setting ${key}: ...`)
      }
    },
    {
      name: 'set',
      description: 'Set a setting',
      options: [
        { name: 'key', description: 'Setting key', type: 'string', required: true },
        { name: 'value', description: 'Setting value', type: 'string', required: true }
      ],
      run: async (ctx) => {
        const { key, value } = ctx.options as { key: string; value: string }
        await ctx.reply(`Set ${key} = ${value}`)
      }
    }
  ]
})

createBot({ slashCommands: [settings] })
```

### Rich Replies

```ts
await ctx.reply({
  content: "Here's some info",
  embeds: [
    {
      title: 'Status Report',
      description: 'All systems operational',
      color: 0x00ff00,
      fields: [
        { name: 'Uptime', value: '99.9%', inline: true },
        { name: 'Latency', value: '42ms', inline: true }
      ],
      footer: { text: 'Last updated' },
      timestamp: new Date().toISOString()
    }
  ],
  attachments: [
    { url: 'https://example.com/report.csv', filename: 'report.csv' }
  ],
  allowedMentions: { parse: ['users'], repliedUser: false },
  ephemeral: true // Only for slash commands
})
```

### Embed Builder

```ts
const myEmbed = embed()
  .setTitle('Hello!')
  .setDescription('This is a rich embed')
  .setColor(0x5865f2)
  .addField('Field 1', 'Value 1', true)
  .addField('Field 2', 'Value 2', true)
  .setFooter('Powered by Flora')
  .toJSON()

await ctx.reply({ embeds: [myEmbed] })
```

## KV Store API

Persistent key-value storage scoped per guild.

```ts
// Get a named store
const store = kv.store('mydata')

// Basic operations
await store.set('key', 'value')
const value = await store.get('key') // string | null
await store.delete('key')

// With metadata and expiration
await store.set('session', JSON.stringify(data), {
  expiration: Math.floor(Date.now() / 1000) + 3600, // 1 hour
  metadata: { userId: '123' }
})

// Get with metadata
const { value, metadata } = await store.getWithMetadata('session')

// List keys with pagination
const result = await store.list({ prefix: 'user:', limit: 100 })
for (const key of result.keys) {
  console.log(key.name, key.metadata)
}
if (!result.list_complete) {
  const nextPage = await store.list({ cursor: result.cursor })
}
```

## Utility Functions

```ts
// Check if user has a role
if (hasRole(ctx, '123456789')) {
  // ...
}

// Get subcommand name
const sub = getSubcommand(ctx) // "get" | "set" | undefined

// Get subcommand group
const group = getSubcommandGroup(ctx)
```

## Types

The SDK exports types auto-generated from Rust:

### Event Payloads

- `UserPayload` - Discord user data
- `MemberPayload` - Guild member data
- `MessagePayload` - Message event data
- `MessageUpdatePayload` - Message update event data
- `MessageDeletePayload` - Message delete event data
- `InteractionCreatePayload` - Slash command interaction data
- `ReadyPayload` - Bot ready event data

### Context Types

- `MessageContext` - Context for message events
- `MessageUpdateContext` - Context for message update events
- `InteractionContext` - Context for slash command interactions

### Command Types

- `Command` - Prefix command definition
- `SlashCommand` - Slash command definition
- `SlashCommandOption` - Slash command option
- `SlashSubcommand` - Slash command subcommand

### KV Types

- `ListKeysResult` - Result from `kv.store().list()`
- `KvKeyInfo` - Key information including metadata

## Development

```bash
# Generate TypeScript types from Rust
./scripts/generate-types.sh

# Build the SDK bundle
bun run sdk/build.ts

# Run tests
cd sdk && bun test
```

## Architecture

```
Rust Structs (src/)
    ↓ ts-rs derives
sdk/src/generated/*.ts (wire format types)
    ↓ imported by
sdk/src/index.ts (SDK runtime + type aliases)
    ↓ bundled by rolldown
dist/sdk-bundle.js (IIFE for V8 runtime)
```

Types are automatically generated when running `cargo test`. The generated files in `sdk/src/generated/` should be committed to version control.
