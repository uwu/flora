import { afterEach, beforeEach, describe, expect, it } from 'vite-plus/test'
import { createBot, prefix, slash } from '../sdk/commands'
import { store } from '../sdk/kv'
import { TestHarness } from './harness'

describe('TestHarness', () => {
  const t = new TestHarness()

  afterEach(() => {
    t.teardown()
  })

  describe('prefix commands', () => {
    beforeEach(() => {
      t.setup(() => {
        createBot({
          prefix: '!',
          commands: [
            prefix({
              name: 'ping',
              run: async (ctx) => {
                await ctx.reply('pong')
              }
            }),
            prefix({
              name: 'echo',
              run: async (ctx) => {
                await ctx.reply(ctx.args.join(' '))
              }
            })
          ]
        })
      })
    })

    it('dispatches prefix command and records reply', async () => {
      const r = await t.message({ content: '!ping' })
      expect(r.replies).toHaveLength(1)
      expect(r.firstReply).toMatchObject({ content: 'pong' })
    })

    it('passes args to prefix command', async () => {
      const r = await t.message({ content: '!echo hello world' })
      expect(r.replies).toHaveLength(1)
      expect(r.firstReply).toMatchObject({ content: 'hello world' })
    })

    it('ignores messages without prefix', async () => {
      const r = await t.message({ content: 'hello' })
      expect(r.replies).toHaveLength(0)
    })

    it('ignores bot messages', async () => {
      const r = await t.message({ content: '!ping', author: { bot: true } })
      expect(r.replies).toHaveLength(0)
    })
  })

  describe('slash commands', () => {
    beforeEach(() => {
      t.setup(() => {
        createBot({
          slashCommands: [
            slash({
              name: 'greet',
              description: 'Greet someone',
              run: async (ctx) => {
                await ctx.reply(`Hello, ${ctx.options.name}!`)
              }
            })
          ]
        })
      })
    })

    it('dispatches slash command with options', async () => {
      const r = await t.interaction('greet', { name: 'World' })
      expect(r.replies).toHaveLength(1)
      expect(r.firstReply).toMatchObject({ content: 'Hello, World!' })
    })

    it('records interaction response op', async () => {
      const r = await t.interaction('greet', { name: 'Test' })
      expect(r.replies[0]!.op).toBe('op_send_interaction_response')
    })
  })

  describe('component interactions', () => {
    beforeEach(() => {
      t.setup(() => {
        on('componentInteraction', async (ctx) => {
          const data = ctx.msg.data as any
          await ctx.reply(`clicked: ${data?.custom_id}`)
        })
      })
    })

    it('dispatches component interaction', async () => {
      const r = await t.componentInteraction('btn-confirm')
      expect(r.replies).toHaveLength(1)
      expect(r.firstReply).toMatchObject({ content: 'clicked: btn-confirm' })
    })
  })

  describe('modal submit', () => {
    beforeEach(() => {
      t.setup(() => {
        on('modalSubmit', async (ctx) => {
          const data = ctx.msg.data as any
          await ctx.reply(`modal: ${data?.custom_id}`)
        })
      })
    })

    it('dispatches modal submit', async () => {
      const r = await t.modalSubmit('feedback-form', { message: 'great' })
      expect(r.replies).toHaveLength(1)
      expect(r.firstReply).toMatchObject({ content: 'modal: feedback-form' })
    })
  })

  describe('KV operations', () => {
    beforeEach(() => {
      t.setup(() => {
        createBot({
          prefix: '!',
          commands: [
            prefix({
              name: 'save',
              run: async (ctx) => {
                const s = store('data')
                await s.set('key1', 'value1')
                await ctx.reply('saved')
              }
            }),
            prefix({
              name: 'load',
              run: async (ctx) => {
                const s = store('data')
                const val = await s.get('key1')
                await ctx.reply(val ?? 'not found')
              }
            })
          ]
        })
      })
    })

    it('stores and retrieves KV data', async () => {
      await t.message({ content: '!save' })
      const r = await t.message({ content: '!load' })
      expect(r.firstReply).toMatchObject({ content: 'value1' })
    })

    it('returns null for missing keys', async () => {
      const r = await t.message({ content: '!load' })
      expect(r.firstReply).toMatchObject({ content: 'not found' })
    })

    it('exposes kv store for direct assertions', async () => {
      await t.message({ content: '!save' })
      expect(t.kv.get('data', 'key1')).toBe('value1')
    })

    it('supports delete', async () => {
      await t.message({ content: '!save' })
      t.kv.delete('data', 'key1')
      const r = await t.message({ content: '!load' })
      expect(r.firstReply).toMatchObject({ content: 'not found' })
    })

    it('supports list keys', () => {
      t.kv.set('data', 'a', '1', {})
      t.kv.set('data', 'b', '2', {})
      t.kv.set('data', 'c', '3', {})
      const result = t.kv.listKeys({ prefix: 'a' }, 'data')
      expect(result.keys).toHaveLength(1)
      expect(result.keys[0]!.name).toBe('a')
      expect(result.listComplete).toBe(true)
    })
  })

  describe('cron', () => {
    it('triggers cron handler', async () => {
      let called = false
      t.setup(() => {
        cron('cleanup', '0 * * * *', async () => {
          called = true
        })
      })
      await t.triggerCron('cleanup')
      expect(called).toBe(true)
    })

    it('records cron registration op', () => {
      t.setup(() => {
        cron('daily', '0 0 * * *', () => {})
      })
      const calls = t.allCalls().filter((c) => c.op === 'op_register_cron')
      expect(calls).toHaveLength(1)
      expect(calls[0]!.args[0]).toMatchObject({ name: 'daily', expr: '0 0 * * *' })
    })
  })

  describe('secrets', () => {
    it('returns configured secrets', () => {
      const h = new TestHarness({ secrets: { TOKEN: 'abc123' } })
      h.setup(() => {})
      expect(secrets.get('TOKEN')).toBe('abc123')
      h.teardown()
    })

    it('supports setSecret at runtime', () => {
      t.setup(() => {})
      t.setSecret('API_KEY', 'xyz')
      expect(secrets.get('API_KEY')).toBe('xyz')
    })
  })

  describe('rest calls', () => {
    beforeEach(() => {
      t.setup(() => {
        on('messageCreate', async (ctx) => {
          await ctx.reply('hi')
          await ctx.edit('edited')
        })
      })
    })

    it('records send and edit ops', async () => {
      const r = await t.message({ content: 'test' })
      expect(r.replies).toHaveLength(1)
      expect(r.edits).toHaveLength(1)
    })
  })

  describe('reset', () => {
    it('clears state and allows re-setup', async () => {
      t.setup(() => {
        createBot({
          prefix: '!',
          commands: [prefix({ name: 'a', run: async (ctx) => ctx.reply('a') })]
        })
      })

      await t.message({ content: '!a' })
      expect(t.allCalls().length).toBeGreaterThan(0)

      t.reset()

      t.setup(() => {
        createBot({
          prefix: '!',
          commands: [prefix({ name: 'b', run: async (ctx) => ctx.reply('b') })]
        })
      })

      const r = await t.message({ content: '!b' })
      expect(r.firstReply).toMatchObject({ content: 'b' })

      // old command should not work
      const r2 = await t.message({ content: '!a' })
      expect(r2.replies).toHaveLength(0)
    })
  })

  describe('multiple dispatches', () => {
    beforeEach(() => {
      t.setup(() => {
        createBot({
          prefix: '!',
          commands: [prefix({ name: 'x', run: async (ctx) => ctx.reply('x') })]
        })
      })
    })

    it('accumulates allCalls across dispatches', async () => {
      await t.message({ content: '!x' })
      await t.message({ content: '!x' })
      const all = t.allCalls().filter((c) => c.op === 'op_send_message')
      expect(all).toHaveLength(2)
    })

    it('returns per-dispatch results', async () => {
      const r1 = await t.message({ content: '!x' })
      const r2 = await t.message({ content: '!x' })
      expect(r1.replies).toHaveLength(1)
      expect(r2.replies).toHaveLength(1)
    })
  })

  describe('mockResponse', () => {
    it('returns mocked value for fetch ops', async () => {
      t.setup(() => {
        on('messageCreate', async (ctx) => {
          const msg = await (globalThis as any).Deno.core.ops.op_fetch_message({
            channelId: '123',
            messageId: '456'
          })
          await ctx.reply(msg.content)
        })
      })

      t.mockResponse('op_fetch_message', { content: 'fetched content' })
      const r = await t.message({ content: 'test' })
      expect(r.firstReply).toMatchObject({ content: 'fetched content' })
    })

    it('supports function mocks', async () => {
      t.setup(() => {
        on('messageCreate', async (ctx) => {
          const msg = await (globalThis as any).Deno.core.ops.op_fetch_message({ messageId: '789' })
          await ctx.reply(msg.id)
        })
      })

      t.mockResponse('op_fetch_message', (input: any) => ({ id: input.messageId, content: 'hi' }))
      const r = await t.message({ content: 'test' })
      expect(r.firstReply).toMatchObject({ content: '789' })
    })
  })

  describe('console.log capture', () => {
    it('captures logs in dispatch result', async () => {
      t.setup(() => {
        on('messageCreate', async () => {
          console.log('debug', 42)
        })
      })

      const r = await t.message({ content: 'test' })
      expect(r.logs).toHaveLength(1)
      expect(r.logs[0]).toEqual(['debug', 42])
    })
  })
})
