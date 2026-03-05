import { Client, GatewayIntentBits, Partials } from 'discord.js'

const prefix = '!'

const client = new Client({
  intents: [
    GatewayIntentBits.Guilds,
    GatewayIntentBits.GuildMessages,
    GatewayIntentBits.MessageContent
  ],
  partials: [Partials.Channel],
  rest: {
    hashSweepInterval: 0,
    handlerSweepInterval: 0
  }
})

client.on('ready', () => {
  console.log(`discord.js ready as ${client.user?.tag ?? 'unknown'}`)
})

client.on('messageCreate', async (message) => {
  if (message.author.bot) return
  if (!message.content.startsWith(prefix)) return

  const body = message.content.slice(prefix.length).trim()
  if (!body.length) return

  const [command, ...args] = body.split(/\s+/)

  if (command === 'ping') {
    await message.reply('pong (discord.js)')
    return
  }

  if (command === 'echo') {
    const text = args.join(' ').trim()
    await message.reply(text || 'usage: !echo <text>')
  }
})

client.login('__FLORA_THIRDPARTY_DISCORD_TOKEN__').catch((err) => {
  console.log('discord.js login failed', err)
})
