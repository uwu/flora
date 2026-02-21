import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createBot, slash } from './sdk/commands'
import type { InteractionContext } from './sdk/types'

describe('createBot slash commands', () => {
  beforeEach(() => {
    // @ts-expect-error test-only reset
    globalThis.__floraCreateBotState = undefined
  })

  it('routes interactionCreate to matching slash command', async () => {
    const handlers: Record<string, (ctx: InteractionContext) => void | Promise<void>> = {}
    // mock global on
    globalThis.on = (event: string, handler: (ctx: InteractionContext) => void | Promise<void>) => {
      handlers[event] = handler
    }

    const run = vi.fn(async (ctx: InteractionContext) => {
      expect(ctx.msg.commandName).toBe('ping')
      expect(ctx.options).toEqual({ text: 'hello', count: 2 })
      await ctx.reply({ content: 'pong', ephemeral: true })
    })

    createBot({ slashCommands: [slash({ name: 'ping', description: 'Ping command', run })] })

    const reply = vi.fn(() => Promise.resolve())
    const handler = handlers['interactionCreate']
    expect(handler).toBeTruthy()

    await handler!({
      msg: {
        interaction_id: '123',
        interaction_token: 'token',
        application_id: 'app',
        commandName: 'ping',
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
