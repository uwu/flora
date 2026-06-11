---
title: 'Utilities'
description: 'Use helper functions, constants, secrets, and logging'
outline: deep
---

# Utilities

The SDK includes a few helpers for common bot tasks. Like the rest of the runtime SDK, these are available globally in your bot script.

## Role Checking

Use `hasRole()` to check whether the user who ran an interaction has a role.

```typescript
const admin = slash({
  name: 'admin',
  description: 'Admin-only command',
  run: async (ctx) => {
    if (!hasRole(ctx, '123456789012345678')) {
      await ctx.reply({
        content: 'You need the Admin role!',
        ephemeral: true
      })
      return
    }

    await ctx.reply('Admin action performed')
  }
})
```

Signature:

```typescript
function hasRole(ctx: InteractionContext, roleId: string): boolean
```

| Parameter | Type                 | Notes                                           |
| --------- | -------------------- | ----------------------------------------------- |
| `ctx`     | `InteractionContext` | Slash command or component interaction context. |
| `roleId`  | `string`             | Discord role ID to check for.                   |

:::info
`hasRole()` returns `false` when member data is unavailable or the role is missing.
:::

## Subcommand Helpers

### Get Subcommand Name

Use `getSubcommand()` when you want to inspect the raw interaction.

```typescript
on('interactionCreate', async (ctx) => {
  const sub = getSubcommand(ctx)
  console.log('subcommand', sub)
})
```

### Get Subcommand Group

```typescript
on('interactionCreate', async (ctx) => {
  const group = getSubcommandGroup(ctx)

  if (group) {
    console.log('group', group)
  }
})
```

Signatures:

```typescript
function getSubcommand(ctx: InteractionContext): string | undefined
function getSubcommandGroup(ctx: InteractionContext): string | undefined
```

## Examples

### Role-Based Access Control

```typescript
const ADMIN_ROLE = '123456789012345678'
const MOD_ROLE = '234567890123456789'

const modAction = slash({
  name: 'mod',
  description: 'Moderator command',
  run: async (ctx) => {
    if (!hasRole(ctx, ADMIN_ROLE) && !hasRole(ctx, MOD_ROLE)) {
      await ctx.reply({
        content: 'You need the Moderator or Admin role!',
        ephemeral: true
      })
      return
    }

    await ctx.reply('Moderation action completed')
  }
})
```

### Dynamic Subcommand Logging

```typescript
const settings = slash({
  name: 'settings',
  description: 'Manage settings',
  subcommands: [
    {
      name: 'get',
      description: 'Get a setting',
      options: [{ name: 'key', description: 'Key', type: 'string', required: true }],
      run: async (ctx) => {
        const key = ctx.options.key as string
        const store = kv.store('settings')
        const value = await store.get(key)
        await ctx.reply(`${key}: ${value || 'not set'}`)
      }
    },
    {
      name: 'set',
      description: 'Set a setting',
      options: [
        { name: 'key', description: 'Key', type: 'string', required: true },
        { name: 'value', description: 'Value', type: 'string', required: true }
      ],
      run: async (ctx) => {
        const { key, value } = ctx.options as { key: string; value: string }
        const store = kv.store('settings')
        await store.set(key, value)
        await ctx.reply(`Set ${key} = ${value}`)
      }
    }
  ]
})

on('interactionCreate', async (ctx) => {
  if (ctx.msg.commandName === 'settings') {
    const sub = getSubcommand(ctx)
    console.log(`user called /settings ${sub}`)
  }
})
```

### Permission Guard

```typescript
const REQUIRED_ROLES = ['123456789', '234567890', '345678901']

function requireAnyRole(ctx: InteractionContext, roleIds: string[]): boolean {
  return roleIds.some((id) => hasRole(ctx, id))
}

const restricted = slash({
  name: 'restricted',
  description: 'Restricted command',
  run: async (ctx) => {
    if (!requireAnyRole(ctx, REQUIRED_ROLES)) {
      await ctx.reply({
        content: 'You do not have permission!',
        ephemeral: true
      })
      return
    }

    await ctx.reply('Access granted')
  }
})
```

## Built-In Constants

Constants are global in the runtime. You can use them directly in bot scripts.

### Button Styles

```typescript
ButtonStyle.Primary
ButtonStyle.Secondary
ButtonStyle.Success
ButtonStyle.Danger
```

`ButtonStyles` is also available as an alias:

```typescript
ButtonStyles.Primary
```

### Input Text Styles

```typescript
InputTextStyles.Short
InputTextStyles.Paragraph
```

`InputTextStyle` is also available if you prefer the singular name.

### Component Types

```typescript
ComponentType.ActionRow
ComponentType.Button
ComponentType.StringSelect
ComponentType.InputText
ComponentType.UserSelect
ComponentType.RoleSelect
ComponentType.MentionableSelect
ComponentType.ChannelSelect
ComponentType.Container
ComponentType.Label
ComponentType.FileUpload
```

### Message Flags

Use message flags when Discord requires them, like components v2:

```typescript
await ctx.reply({
  components: [container().addComponents(textDisplay('Hello')).toJSON()],
  flags: MessageFlags.IS_COMPONENTS_V2
})
```

## Secrets Management

Use `secrets.get()` for values that should not live in your source code.

```typescript
const apiKey = secrets.get('API_KEY')

if (!apiKey) {
  console.log('API_KEY not configured')
} else {
  await fetch('https://api.example.com', {
    headers: { Authorization: `Bearer ${apiKey}` }
  })
}
```

Signature:

```typescript
interface Secrets {
  get(name: string): string | undefined
}

declare const secrets: Secrets
```

:::info
Secrets are configured per deployment. They are not stored in your bot script.
:::

## Logging

Use `console.log()` for debugging and monitoring.

```typescript
console.log('bot started')
console.log('user id', ctx.msg.author?.id)
console.log('command', ctx.msg.commandName)
```

:::tip
flora captures console output and makes it available in runtime logs. Only `console.log()` is supported.
:::

## Type Imports

Most runtime code can use globals. If you want explicit type imports in local tooling, import them from `@uwu/flora-sdk`:

```typescript
import type { InteractionContext, MessageContext } from '@uwu/flora-sdk'
```

## Tips

- Check permissions at the start of command handlers.
- Use ephemeral replies for permission errors and other private feedback.
- Log important command usage and expected errors.
- Keep secrets out of source code and read them at runtime.
