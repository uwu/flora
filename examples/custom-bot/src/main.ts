createBot({
  prefix: '!',
  slashCommands: [
    slash({
      name: 'ping',
      description: 'Reply with pong',
      async run(ctx) {
        await ctx.reply('pong')
      }
    })
  ],
  commands: [
    prefix({
      name: 'ping',
      async run(ctx) {
        await ctx.reply('pong from a guild or DM')
      }
    })
  ]
})

console.log(`custom bot runtime ${customBot.id} loaded`)
