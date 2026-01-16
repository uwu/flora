---
outline: deep
---

# Examples

This page covers the common flows end-to-end: prefix commands, slash commands,
subcommands, embeds, KV usage, and deployment via the CLI.

## Minimal bot

```ts
on('ready', async () => console.log('ready'))

const ping = prefix({
  name: 'ping',
  description: 'pong',
  run: async (ctx) => ctx.reply('pong')
})

createBot({ prefix: '!', commands: [ping] })
```

## Prefix command with args

```ts
const math = prefix({
  name: 'add',
  description: 'Add two numbers',
  run: async (ctx) => {
    const [a, b] = ctx.args.map((x) => Number(x))
    await ctx.reply(`sum: ${a + b}`)
  }
})

createBot({ prefix: '!', commands: [math] })
```

## Slash command

```ts
const echo = slash({
  name: 'echo',
  description: 'Echo text',
  options: [
    { name: 'text', description: 'Text to echo', type: 'string', required: true }
  ],
  run: async (ctx) => {
    const text = ctx.options.text as string
    await ctx.reply({ content: text, ephemeral: true })
  }
})

createBot({ slashCommands: [echo] })
```

## Slash subcommands

```ts
const notes = slash({
  name: 'notes',
  description: 'Manage notes',
  subcommands: [
    {
      name: 'get',
      description: 'Get a note',
      options: [{ name: 'key', description: 'Key', type: 'string', required: true }],
      run: async (ctx) => {
        const key = ctx.options.key as string
        const store = kv.store('notes')
        const value = await store.get(key)
        await ctx.reply(value ?? 'missing')
      }
    },
    {
      name: 'set',
      description: 'Set a note',
      options: [
        { name: 'key', description: 'Key', type: 'string', required: true },
        { name: 'value', description: 'Value', type: 'string', required: true }
      ],
      run: async (ctx) => {
        const { key, value } = ctx.options as { key: string; value: string }
        const store = kv.store('notes')
        await store.set(key, value)
        await ctx.reply(`saved ${key}`)
      }
    }
  ]
})

createBot({ slashCommands: [notes] })
```

## Embeds

```ts
const info = embed()
  .setTitle('Build info')
  .setDescription('Nightly deployment')
  .setColor(0x3366ff)
  .addField('Region', 'us-east', true)
  .addField('Version', 'v0.1.0', true)
  .toJSON()

await ctx.reply({ embeds: [info] })
```

## Components

```ts
const row = actionRow().addComponents(
  button().setCustomId('primary').setLabel('Primary').setStyle(ButtonStyle.Primary),
  button().setCustomId('danger').setLabel('Danger').setStyle(ButtonStyle.Danger)
)

await ctx.reply({ content: 'Buttons', components: [row.toJSON()] })
```

## KV store

```ts
const prefs = kv.store('prefs')

await prefs.set('color', 'green', {
  metadata: { source: 'setup' }
})

const color = await prefs.get('color')
await ctx.reply(`color: ${color}`)

const page = await prefs.list({ prefix: 'user:', limit: 100 })
```

## Deploy with the CLI

1. Save your script (for example `src/main.ts`).
2. Login once:

```bash
flora login <token>
```

3. Deploy to a guild:

```bash
flora deploy --guild 123456789012345678 src/main.ts --root src
```

4. Check the deployment:

```bash
flora get --guild 123456789012345678
```

## Logs

```bash
flora logs --guild 123456789012345678 --limit 200
flora logs --guild 123456789012345678 --follow
```

## KV via CLI

```bash
flora kv create-store --guild 123456789012345678 --name notes
flora kv set --guild 123456789012345678 --store notes --key welcome "hi there"
flora kv get --guild 123456789012345678 --store notes welcome
flora kv list-keys --guild 123456789012345678 --store notes
```
