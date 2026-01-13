import { describe, expect, it, mock } from 'bun:test'
import type { InteractionContext } from './index'
import { createBot, slash } from './index'

describe('createBot slash commands', () => {
  it('routes interactionCreate to matching slash command', async () => {
    const handlers: Record<string, (ctx: InteractionContext) => void | Promise<void>> = {}
    // mock global on
    // @ts-expect-error
    globalThis.on = (event: string, handler: (ctx: InteractionContext) => void | Promise<void>) => {
      handlers[event] = handler
    }

    const run = mock(async (ctx: InteractionContext) => {
      expect(ctx.msg.commandName).toBe('ping')
      expect(ctx.options).toEqual({ text: 'hello', count: 2 })
      await ctx.reply({ content: 'pong', ephemeral: true })
    })

    createBot({ slashCommands: [slash({ name: 'ping', description: 'Ping command', run })] })

    const reply = mock(() => Promise.resolve())
    const handler = handlers['interactionCreate']
    expect(handler).toBeTruthy()

    await handler!({
      msg: {
        interaction_id: '123',
        interaction_token: 'token',
        application_id: 'app',
        command_name: 'ping',
        data: {
          options: [
            { name: 'text', value: 'hello' },
            { name: 'count', value: 2 }
          ]
        },
        user: { id: 'u', username: 'u', bot: false }
      },
      reply
    } as unknown as InteractionContext)

    expect(run).toHaveBeenCalled()
    expect(reply).toHaveBeenCalledWith({ content: 'pong', ephemeral: true })
  })
})
