---
title: 'Commands'
description: 'Define prefix commands and slash commands'
outline: deep
---

# Commands

flora supports two command styles:

- Prefix commands run when a user sends a message that starts with your bot prefix, like `!ping`.
- Slash commands are registered with Discord and appear in the command picker, like `/hello`.

You can use either style, or both, in the same bot.

## Prefix Commands

Prefix commands are the quickest way to handle message-based commands.

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

Users trigger this command with `!ping`.

## Command Arguments

Prefix command arguments are available through `ctx.args`.

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

With a `!` prefix, `!greet Alice` replies with `Hello, Alice!`.

## Prefix Context

| Property    | Type           | What it gives you                                                  |
| ----------- | -------------- | ------------------------------------------------------------------ |
| `ctx.msg`   | `EventMessage` | The Discord message payload.                                       |
| `ctx.args`  | `string[]`     | Space-separated arguments after the command name.                  |
| `ctx.reply` | `function`     | Sends a reply. Accepts a string or a reply options object.         |
| `ctx.edit`  | `function`     | Edits the original message or interaction response when available. |

## Slash Commands

Slash commands are registered with Discord. They are better when you want discoverable commands, typed options, or private responses.

```typescript
const hello = slash({
  name: 'hello',
  description: 'Say hello',
  run: async (ctx) => {
    await ctx.reply({ content: 'Hello!', ephemeral: true })
  }
})

createBot({
  slashCommands: [hello]
})
```

## Options

Slash command options are declared on the command and then read from `ctx.options`.

```typescript
const echo = slash({
  name: 'echo',
  description: 'Echo your input',
  options: [
    {
      name: 'text',
      description: 'Text to echo',
      type: 'string',
      required: true
    },
    {
      name: 'loud',
      description: 'Make it loud',
      type: 'boolean',
      required: false
    }
  ],
  run: async (ctx) => {
    const text = ctx.options.text as string
    const loud = ctx.options.loud as boolean
    const output = loud ? text.toUpperCase() : text
    await ctx.reply({ content: output, ephemeral: true })
  }
})
```

### Option Types

| Type      | Example                                                 |
| --------- | ------------------------------------------------------- |
| `string`  | `{ name: 'message', type: 'string', required: true }`   |
| `integer` | `{ name: 'count', type: 'integer', required: false }`   |
| `number`  | `{ name: 'amount', type: 'number', required: false }`   |
| `boolean` | `{ name: 'enabled', type: 'boolean', required: false }` |

## Subcommands

Use subcommands when one slash command owns a few related actions.

```typescript
const settings = slash({
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

Users can call `/settings get key:prefix` or `/settings set key:prefix value:!`.

## Reply Options

`ctx.reply()` accepts a string or a reply options object.

| Option            | Type          | Notes                                                          |
| ----------------- | ------------- | -------------------------------------------------------------- |
| `content`         | `string`      | Message text.                                                  |
| `embeds`          | `RawEmbed[]`  | Embed objects from `embed().toJSON()`.                         |
| `components`      | `JsonValue[]` | Component rows or components v2 objects.                       |
| `ephemeral`       | `boolean`     | For slash commands and interactions, visible only to the user. |
| `allowedMentions` | `object`      | Controls which mentions Discord should parse.                  |
| `flags`           | `number`      | Message flags, like `MessageFlags.IS_COMPONENTS_V2`.           |

## Type Definitions

Prefix commands:

```typescript
export type Command = {
  name: string
  description?: string
  run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
}

export function prefix(command: Command): Command
```

Slash commands:

```typescript
export type SlashCommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
  subcommands?: SlashSubcommand[]
  run?: (ctx: InteractionContext) => Promise<void> | void
}

export function slash(command: SlashCommand): SlashCommand
```

Bot registration:

```typescript
export type CreateOptions = {
  prefix?: string
  commands?: Command[]
  prefixCommands?: Command[]
  slashCommands?: SlashCommand[]
}

export function createBot(options: CreateOptions): void
```

## Counter Example

Here is a slash command with subcommands and KV storage:

```typescript
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
    },
    {
      name: 'reset',
      description: 'Reset the counter',
      run: async (ctx) => {
        const store = kv.store('counters')
        await store.set('main', '0')
        await ctx.reply('Counter reset to 0')
      }
    }
  ]
})

createBot({ slashCommands: [counter] })
```

See [KV Storage](/docs/sdk/kv-storage) for more ways to persist state.

## Tips

- Use short, descriptive command names.
- Validate user input before doing work.
- Wrap risky async work in `try`/`catch` and reply with a useful message.
- Use `ephemeral: true` for private slash command responses.
- Keep `createBot()` at the bottom of the file so all commands are defined first.
