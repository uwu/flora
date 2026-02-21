import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createBot, slash } from './sdk/commands'

describe('createBot slash registration', () => {
  beforeEach(() => {
    // @ts-expect-error test-only reset
    globalThis.__floraCreateBotState = undefined
  })

  it('registers slash commands when guild id is present', () => {
    const onHandlers: Record<string, (ctx: any) => any> = {}
    globalThis.on = (event: string, handler: (ctx: any) => any) => {
      onHandlers[event] = handler
    }

    const register = vi.fn(() => Promise.resolve())
    globalThis.registerSlashCommands = register
    globalThis.__floraGuildId = '123'

    createBot({
      slashCommands: [
        slash({
          name: 'ping',
          description: 'Reply with pong',
          options: [
            { name: 'text', description: 'say something', required: true }
          ],
          run: () => {}
        })
      ]
    })

    expect(register).toHaveBeenCalledWith([
      {
        name: 'ping',
        description: 'Reply with pong',
        options: [
          {
            name: 'text',
            description: 'say something',
            required: true,
            type: undefined
          }
        ]
      }
    ])
  })

  it('skips duplicate createBot registration', () => {
    const register = vi.fn(() => Promise.resolve())
    const on = vi.fn()
    const log = vi.spyOn(console, 'log').mockImplementation(() => {})

    globalThis.on = on as any
    globalThis.registerSlashCommands = register
    globalThis.__floraGuildId = '123'

    const options = {
      slashCommands: [
        slash({
          name: 'ping',
          description: 'Reply with pong',
          run: () => {}
        })
      ]
    }

    createBot(options)
    createBot(options)

    expect(on).toHaveBeenCalledTimes(2)
    expect(register).toHaveBeenCalledTimes(1)
    expect(log).toHaveBeenCalledWith(
      '[flora/sdk] createBot called multiple times; skipping duplicate registration'
    )
    log.mockRestore()
  })
})
