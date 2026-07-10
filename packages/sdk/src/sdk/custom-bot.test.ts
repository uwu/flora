import { beforeEach, describe, expect, it, vi } from 'vite-plus/test'
import { customBot } from './custom-bot'

describe('customBot', () => {
  const createDm = vi.fn(async () => ({ id: 'channel' }))
  const upsertGlobalCommands = vi.fn(async () => undefined)

  beforeEach(() => {
    globalThis.__floraRuntimeKind = 'custom_bot'
    globalThis.__floraBotId = 'bot-id'
    ;(globalThis as any).Deno = {
      core: {
        ops: {
          op_create_dm: createDm,
          op_upsert_global_commands: upsertGlobalCommands
        }
      }
    }
    createDm.mockClear()
    upsertGlobalCommands.mockClear()
  })

  it('exposes the active custom bot id and DM operation', async () => {
    expect(customBot.id).toBe('bot-id')
    await expect(customBot.createDm('user-id')).resolves.toEqual({ id: 'channel' })
    expect(createDm).toHaveBeenCalledWith({ userId: 'user-id' })
  })

  it('rejects custom bot operations in other runtime kinds', () => {
    globalThis.__floraRuntimeKind = undefined
    expect(() => customBot.id).toThrow('custom bot runtime')
  })
})
