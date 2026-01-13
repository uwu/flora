import { describe, expect, it, mock } from 'bun:test'
import type { InteractionContext } from './index'
import { createBot, slash } from './index'

describe('createBot slash registration', () => {
  it('registers slash commands when guild id is present', () => {
    const onHandlers: Record<string, (ctx: any) => any> = {}
    globalThis.on = (event: string, handler: (ctx: any) => any) => {
      onHandlers[event] = handler
    }

    const register = mock(() => Promise.resolve())
    // @ts-expect-error
    globalThis.slash = register
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
})
