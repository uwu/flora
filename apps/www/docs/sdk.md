---
outline: deep
---

# SDK

Use the Flora TypeScript SDK to build bot scripts that run inside the Flora runtime.
In the runtime, SDK functions are available globally (no imports needed).

## Quickstart

```ts
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

## Event handlers

```ts
on('ready', async (ctx) => {
  console.log('bot ready')
})

on('messageCreate', async (ctx) => {
  if (ctx.msg.author?.bot) return
  if (ctx.msg.content === '!hello') {
    await ctx.reply('hello!')
  }
})

on('messageUpdate', async (ctx) => {
  console.log('message updated', ctx.msg.id)
})

on('messageDelete', async (ctx) => {
  console.log('message deleted', ctx.msg.id)
})

on('messageDeleteBulk', async (ctx) => {
  console.log('bulk delete', ctx.msg.ids.length)
})

on('interactionCreate', async (ctx) => {
  console.log('interaction', ctx.msg.commandName)
})
```

## Prefix commands

```ts
const greet = prefix({
  name: 'greet',
  description: 'Greet someone',
  run: async (ctx) => {
    const name = ctx.args[0] || 'world'
    await ctx.reply(`Hello, ${name}!`)
  }
})

createBot({
  prefix: '!',
  commands: [greet]
})
```

## Slash commands

```ts
const echo = slash({
  name: 'echo',
  description: 'Echo your input',
  options: [
    { name: 'text', description: 'Text to echo', type: 'string', required: true }
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

### Slash command subcommands

```ts
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

## Replies and edits

```ts
await ctx.reply('simple reply')

await ctx.reply({
  content: 'rich reply',
  ephemeral: true,
  allowedMentions: { parse: ['users'] }
})

await ctx.edit('edited content')
```

## Embeds

```ts
const status = embed()
  .setTitle('Status')
  .setDescription('All systems nominal')
  .setColor(0x00ff00)
  .addField('Uptime', '99.9%', true)
  .addField('Latency', '42ms', true)
  .setFooter('Flora')
  .toJSON()

await ctx.reply({ embeds: [status] })
```

## Components

```ts
const row = actionRow().addComponents(
  button().setCustomId('ping').setLabel('Ping').setStyle(ButtonStyle.Primary),
  button().setUrl('https://example.com').setLabel('Docs')
)

await ctx.reply({ content: 'Pick one', components: [row.toJSON()] })
```

V2 components require a message flag:

```ts
const card = container()
  .setAccentColor(0x3366ff)
  .addComponents(
    section()
      .addComponents(textDisplay('Flora runtime'))
      .setAccessory(thumbnail('https://example.com/logo.png'))
  )

await ctx.reply({
  components: [card.toJSON()],
  flags: MessageFlags.IS_COMPONENTS_V2
})
```

## KV store

```ts
const store = kv.store('settings')

await store.set('prefix', '!')
const prefixValue = await store.get('prefix')

await store.set('session', JSON.stringify({ userId: '123' }), {
  expiration: Math.floor(Date.now() / 1000) + 3600,
  metadata: { source: 'login' }
})

const { value, metadata } = await store.getWithMetadata('session')

const page = await store.list({ prefix: 'user:', limit: 100 })
if (!page.list_complete) {
  await store.list({ cursor: page.cursor })
}
```

## Cron jobs

Schedule tasks to run at fixed times or intervals using cron expressions. All times are evaluated in UTC.

```ts
// Run every minute
cron('heartbeat', '* * * * *', (ctx) => {
  console.log(`Heartbeat at ${ctx.scheduledAt}`)
})

// Run daily at midnight UTC
cron('daily-cleanup', '0 0 * * *', async (ctx) => {
  console.log(`Running ${ctx.name}`)
  // cleanup logic here
})

// Run every hour at minute 30
cron('hourly-report', '30 * * * *', (ctx) => {
  console.log('Generating hourly report')
})

// Run at 9am on weekdays
cron('weekday-reminder', '0 9 * * 1-5', (ctx) => {
  console.log('Good morning!')
})
```

The cron context provides:

- `name`: The cron job name you specified
- `scheduledAt`: ISO 8601 timestamp of when the job was scheduled to run

### Options

```ts
// Skip execution if previous run is still active
cron('long-task', '*/5 * * * *', async (ctx) => {
  await someLongRunningTask()
}, { skipIfRunning: true })
```

- `skipIfRunning`: If `true`, the job won't start a new execution if the previous one is still running. Default: `false`.

Cron expressions follow the usual standard 5-field format: `minute hour day-of-month month day-of-week`.

Limits:

- Max cron jobs per guild: 32
- Handler timeout: 5 seconds

## Utilities

```ts
if (hasRole(ctx, '123456789')) {
  await ctx.reply('role ok')
}

const sub = getSubcommand(ctx)
const group = getSubcommandGroup(ctx)
```

## Types

Types are automatically generated from the runtime structs using https://github.com/taskylizard/t0x. They are available in the `@uwu/flora-sdk` package, and also globally.
