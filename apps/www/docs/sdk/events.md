---
title: 'Event Handlers'
description: 'Listen for Discord events in flora'
outline: deep
---

# Event Handlers

Use `on()` when your bot should react to Discord events directly. Commands are enough for many bots, but event handlers are useful for logging, moderation, reactions, and custom interaction flows.

## Basic Usage

```typescript
on('ready', async (ctx) => {
  console.log('bot ready')
})

on('messageCreate', async (ctx) => {
  if (ctx.msg.author?.bot) return

  if (ctx.msg.content === '!hello') {
    await ctx.reply('hello!')
  }
})
```

## Available Events

| Event                  | Context                       | When it runs                               |
| ---------------------- | ----------------------------- | ------------------------------------------ |
| `ready`                | `BaseContext<EventReady>`     | The bot connects to Discord.               |
| `messageCreate`        | `MessageContext`              | A message is created.                      |
| `messageUpdate`        | `MessageUpdateContext`        | A message is edited.                       |
| `messageDelete`        | `MessageDeleteContext`        | A message is deleted.                      |
| `messageDeleteBulk`    | `MessageDeleteBulkContext`    | Multiple messages are deleted at once.     |
| `interactionCreate`    | `InteractionContext`          | A slash command or interaction is created. |
| `componentInteraction` | `ComponentInteractionContext` | A button or select menu is used.           |
| `modalSubmit`          | `ModalSubmitContext`          | A modal form is submitted.                 |
| `reactionAdd`          | `ReactionContext`             | A reaction is added to a message.          |
| `reactionRemove`       | `ReactionContext`             | A reaction is removed from a message.      |
| `reactionRemoveEmoji`  | `ReactionContext`             | All reactions for one emoji are removed.   |
| `reactionRemoveAll`    | `ReactionRemoveAllContext`    | All reactions are removed from a message.  |

## Event Context

Every event handler receives a context object:

```typescript
type BaseContext<TPayload> = {
  msg: TPayload
  reply: (content: string | MessageReplyOptions) => Promise<void>
  edit: (content: string | MessageEditOptions) => Promise<void>
}
```

`ctx.msg` is the event payload from Discord. `ctx.reply()` and `ctx.edit()` are available when the event can be connected to a message or interaction response.

## Message Events

### Message Create

```typescript
on('messageCreate', async (ctx) => {
  if (ctx.msg.author?.bot) return

  if (ctx.msg.content?.includes('hello')) {
    await ctx.reply('Hi there!')
  }
})
```

:::tip
Always check `ctx.msg.author?.bot` in message handlers. It keeps your bot from responding to other bots or looping on itself.
:::

### Message Update

```typescript
on('messageUpdate', async (ctx) => {
  console.log('message updated', ctx.msg.id)
})
```

### Message Delete

```typescript
on('messageDelete', async (ctx) => {
  console.log('message deleted', ctx.msg.id)
})
```

### Bulk Delete

```typescript
on('messageDeleteBulk', async (ctx) => {
  console.log('bulk delete', ctx.msg.ids.length)
})
```

## Interaction Events

### Interaction Create

`interactionCreate` fires for slash commands and component interactions.

```typescript
on('interactionCreate', async (ctx) => {
  console.log('interaction', ctx.msg.commandName)
})
```

:::info
Slash commands defined with `slash()` are handled automatically by `createBot()`. Use `interactionCreate` only when you need custom interaction handling.
:::

### Component Interactions

```typescript
on('componentInteraction', async (ctx) => {
  const customId = ctx.msg.data?.custom_id

  if (customId === 'delete_button') {
    await ctx.reply({ content: 'Deleted!', ephemeral: true })
  }
})
```

### Modal Submissions

```typescript
on('modalSubmit', async (ctx) => {
  const customId = ctx.msg.data?.custom_id

  if (customId === 'feedback_form') {
    await ctx.reply({ content: 'Thanks for your feedback!', ephemeral: true })
  }
})
```

## Reaction Events

### Reaction Add

```typescript
on('reactionAdd', async (ctx) => {
  const emoji = ctx.msg.emoji

  if (emoji?.name === 'star') {
    console.log('Star reaction added')
  }
})
```

### Reaction Remove

```typescript
on('reactionRemove', async (ctx) => {
  console.log('reaction removed', ctx.msg.emoji?.name)
})
```

### Reaction Remove All

```typescript
on('reactionRemoveAll', async (ctx) => {
  console.log('all reactions removed from message', ctx.msg.messageId)
})
```

## Ready Event

The `ready` event runs when the bot connects.

```typescript
on('ready', async (ctx) => {
  console.log('bot is ready')
})
```

This is a good place to log startup or initialize in-memory resources.

## Multiple Handlers

You can register more than one handler for the same event. flora runs them in registration order.

```typescript
on('messageCreate', async (ctx) => {
  console.log('message from', ctx.msg.author?.username)
})

on('messageCreate', async (ctx) => {
  if (ctx.msg.content?.includes('spam')) {
    await ctx.reply('Please keep the channel clean.')
  }
})
```

## Examples

### Auto-Responder

```typescript
on('messageCreate', async (ctx) => {
  if (ctx.msg.author?.bot) return

  const content = ctx.msg.content?.toLowerCase()

  if (content?.includes('good morning')) {
    await ctx.reply('Good morning!')
  } else if (content?.includes('good night')) {
    await ctx.reply('Good night!')
  }
})
```

### Reaction Role Shell

```typescript
on('reactionAdd', async (ctx) => {
  const emoji = ctx.msg.emoji?.name
  const messageId = ctx.msg.messageId

  if (messageId === 'YOUR_ROLE_MESSAGE_ID') {
    if (emoji === 'red') {
      // Grant red role
    } else if (emoji === 'blue') {
      // Grant blue role
    }
  }
})

on('reactionRemove', async (ctx) => {
  // Remove role when reaction is removed
})
```

### Message Logger

```typescript
const store = kv.store('message_log')

on('messageCreate', async (ctx) => {
  if (ctx.msg.author?.bot) return

  const logEntry = JSON.stringify({
    id: ctx.msg.id,
    author: ctx.msg.author?.username,
    content: ctx.msg.content,
    timestamp: new Date().toISOString()
  })

  await store.set(`msg:${ctx.msg.id}`, logEntry)
})

on('messageDelete', async (ctx) => {
  const logged = await store.get(`msg:${ctx.msg.id}`)

  if (logged) {
    console.log('deleted message', logged)
  }
})
```

## Tips

- Filter bot messages in `messageCreate` handlers.
- `await` async work so handlers complete in a predictable order.
- Catch expected errors and reply with something useful.
- Keep event handlers quick. Use KV or cached state for heavier workflows.
