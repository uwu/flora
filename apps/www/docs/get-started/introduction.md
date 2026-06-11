---
title: Introduction
description: Build Discord bots with TypeScript in a secure, isolated runtime
---

# What is flora?

flora is a fast and secure runtime that lets you write Discord bots for your servers using TypeScript, without worrying about infrastructure. Deploy scripts on a per-guild basis through a single bot, powered by V8 isolates for complete isolation between guilds.

:::info
flora is in early alpha. Expect breaking changes and rough edges as the foundations solidify.
:::

## Key Features

- Each guild runs in its own V8 isolate with enforced resource limits and complete separation.
- TypeScript SDK with global namespace - no imports needed, just write code and deploy it.
- Rich SDK support for both prefix and slash commands with subcommands and options.
- KV storage with expiration, metadata, and prefix filtering out of the box.
- Schedule tasks with cron expressions for scheduled work.
- Deploy, manage logs, and interact with KV storage through the flora CLI

## How It Works

flora provides a runtime that:

1. **Isolates guild scripts** - Each guild's bot script runs in its own V8 isolate with strict resource limits
2. **Handles Discord events** - The runtime receives Discord gateway events and routes them to your handlers
3. **Exposes a clean SDK** - Write TypeScript with global functions with: `on()`, `slash()`, and `kv.store()`
4. **Manages deployment** - Use the flora CLI to bundle and deploy your scripts to specific guilds

```typescript
// Your entire bot in one file
const ping = prefix({
  name: 'ping',
  description: 'Respond with pong',
  run: async (ctx) => {
    await ctx.reply('pong!')
  }
})

createBot({
  prefix: '!',
  commands: [ping]
})
```

## Community & Support

flora is open source and under active development. Any issues or questions? Feel free to open a GitHub issue or join our [discord server](https://discord.gg/dRGTU7n4dC)!
