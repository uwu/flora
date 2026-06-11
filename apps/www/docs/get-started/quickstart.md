---
title: 'Quickstart'
description: 'Get your first flora bot running in minutes'
---

# Quickstart

This guide will walk you through creating and deploying your first guild deployment with flora.

## Prerequisites

Before you begin, make sure you have:

- Node.js installed for local development
- pnpm v12.x

## Create Your First Bot

1. Create a new directory for your guild and initialize a package.json:

```bash
mkdir my-server
cd my-server
pnpm init
```

2. Add the flora SDK as a dev dependency:

```bash
pnpm install -D @uwu/flora-sdk
```

3. Create a file called `src/main.ts` with a simple ping command:

```typescript
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

:::tip
You will see there are no imports, that's because no imports needed! All flora SDK functions are available globally in the runtime.
:::

4. Create a `tsconfig.json` to get type checking:

```json
{
  "extends": "@uwu/flora-sdk/tsconfig",
  "compilerOptions": {
    "outDir": "dist"
  },
  "include": ["src/**/*"]
}
```

5.Use the flora CLI to deploy your bot to a guild:

```bash
flora deploy --guild-id YOUR_GUILD_ID
```

The CLI will deploy it to the runtime. You may not worry about bundling, we install your dependencies with pnpm, and bundle your code with [rolldown](https://rolldown.rs)when you deploy.

Now, in your Discord server, type `!ping` and your bot should respond with "pong!"

## Add More Features

Let's enhance your bot with a slash command and KV storage:

```typescript
const ping = prefix({
  name: 'ping',
  description: 'Respond with pong',
  run: async (ctx) => {
    await ctx.reply('pong!')
  }
})

const hello = slash({
  name: 'hello',
  description: 'Say hello',
  options: [
    {
      name: 'name',
      description: 'Who to greet',
      type: 'string',
      required: false
    }
  ],
  run: async (ctx) => {
    const name = (ctx.options.name as string) || 'world'
    await ctx.reply({
      content: `Hello, ${name}!`,
      ephemeral: true
    })
  }
})

const counter = slash({
  name: 'counter',
  description: 'A simple counter using KV storage',
  subcommands: [
    {
      name: 'get',
      description: 'Get current count',
      run: async (ctx) => {
        const store = kv.store('counters')
        const count = await store.get('main')
        await ctx.reply(`Current count: ${count || 0}`)
      }
    },
    {
      name: 'increment',
      description: 'Increment the counter',
      run: async (ctx) => {
        const store = kv.store('counters')
        const current = parseInt((await store.get('main')) || '0', 10)
        const newCount = current + 1
        await store.set('main', String(newCount))
        await ctx.reply(`Count is now: ${newCount}`)
      }
    }
  ]
})

createBot({
  prefix: '!',
  commands: [ping],
  slashCommands: [hello, counter]
})
```

Deploy the updated bot:

```bash
flora deploy --guild-id YOUR_GUILD_ID
```

And that's the end of this quick start guide! We have have learned:

- Creating prefix and slash commands with options and subcommands
- Storing and retrieving data with the built-in KV store
- Listening for Discord events like messages and reactions
- Creating rich embeds with colors, fields, and images

Next, you may checkout the features available in flora, from the sidebar, happy coding!
