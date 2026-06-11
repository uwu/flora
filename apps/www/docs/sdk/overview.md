---
title: 'SDK Overview'
description: 'Build Discord bots with the flora TypeScript SDK'
outline: deep
---

# SDK Overview

The flora SDK gives you the helpers you need to write a Discord bot in TypeScript. In the flora runtime, the SDK functions are available globally, so your bot files can stay small and focused.

:::tip
You install the SDK for TypeScript types and editor help. When your bot runs in flora, helpers like `prefix()`, `slash()`, `on()`, and `kv.store()` are already available globally.
:::

## Installation

Add the SDK as a dev dependency:

```bash
pnpm install -D @uwu/flora-sdk
```

Then extend the SDK `tsconfig` in your project:

```json
{
  "extends": "@uwu/flora-sdk/tsconfig",
  "compilerOptions": {
    "outDir": "dist"
  },
  "include": ["src/**/*"]
}
```

## Quickstart

Create a `src/main.ts` file with a small prefix command:

```typescript
const ping = prefix({
  name: 'ping',
  description: 'Respond with pong',
  run: async (ctx) => {
    const extra = ctx.args.join(' ') || 'none'
    await ctx.reply(`pong! args: ${extra}`)
  }
})

createBot({
  prefix: '!',
  commands: [ping]
})
```

Deploy it with the CLI:

```bash
flora deploy --guild-id YOUR_GUILD_ID
```

## Core Concepts

1. Define commands with `prefix()` and `slash()`.
2. Register your bot once with `createBot()`.
3. Listen for Discord events with `on()`.
4. Store per-guild data with `kv.store()`.
5. Schedule recurring work with `cron()`.
6. Deploy the script to flora, where it runs in an isolated runtime for that guild.

## Global Functions

These helpers are available in bot scripts without imports:

| Helper                       | What it does                                                          |
| ---------------------------- | --------------------------------------------------------------------- |
| `prefix()`                   | Defines a message command triggered by your bot prefix.               |
| `slash()`                    | Defines a Discord slash command.                                      |
| `createBot()`                | Registers commands and configures the bot. Call this once per script. |
| `on()`                       | Registers a handler for a Discord event.                              |
| `embed()`                    | Builds rich Discord embeds.                                           |
| `actionRow()` and `button()` | Build interactive message components.                                 |
| `kv.store()`                 | Opens a per-guild key-value store.                                    |
| `cron()`                     | Schedules recurring jobs with cron expressions.                       |
| `secrets.get()`              | Reads a deployment secret by name.                                    |

## Context Object

Command and event handlers receive a context object. The payload type depends on the command or event you are handling, but the shape is always familiar:

```typescript
type Context = {
  msg: unknown
  reply: (content: string | MessageReplyOptions) => Promise<void>
  edit: (content: string | MessageEditOptions) => Promise<void>
}
```

Use `ctx.msg` for the Discord payload, `ctx.reply()` to send a response, and `ctx.edit()` when you need to update an existing response.

## Reply Options

A plain string is enough for simple replies:

```typescript
await ctx.reply('Hello, world!')
```

Use an object when you want embeds, components, or slash-command-only options like `ephemeral`:

```typescript
await ctx.reply({
  content: 'Check this out',
  embeds: [myEmbed],
  components: [actionRow],
  ephemeral: true
})
```

You can also edit the original response:

```typescript
await ctx.edit('Updated content')
```

## Command Arguments

Prefix commands receive arguments as `ctx.args`:

```typescript
const greet = prefix({
  name: 'greet',
  description: 'Greet someone',
  run: async (ctx) => {
    const name = ctx.args[0] || 'world'
    await ctx.reply(`Hello, ${name}!`)
  }
})
```

Slash commands receive typed options through `ctx.options`:

```typescript
const echo = slash({
  name: 'echo',
  description: 'Echo your input',
  options: [{ name: 'text', description: 'Text to echo', type: 'string', required: true }],
  run: async (ctx) => {
    const text = ctx.options.text as string
    await ctx.reply({ content: text, ephemeral: true })
  }
})
```

## Type Safety

Most bot scripts do not need imports, but you can import types when you want stronger annotations:

```typescript
import type { MessageContext, InteractionContext, SlashCommand, Command } from '@uwu/flora-sdk'
```

:::info
The type definitions are generated from the runtime and shipped with `@uwu/flora-sdk`, so editor feedback should match what the runtime exposes.
:::

## Next Steps

- [Commands](/docs/sdk/commands) covers prefix commands, slash commands, options, and subcommands.
- [Events](/docs/sdk/events) shows how to listen for Discord gateway events.
- [Embeds](/docs/sdk/embeds) walks through rich Discord embeds.
- [Components](/docs/sdk/components) covers buttons, select menus, and components v2.
- [KV Storage](/docs/sdk/kv-storage) shows how to persist data per guild.
- [Cron Jobs](/docs/sdk/cron-jobs) covers scheduled work.
