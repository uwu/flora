import { beforeEach, describe, expect, it } from 'vite-plus/test'
import { defineOrchestrator } from './orchestrator'

describe('defineOrchestrator', () => {
  beforeEach(() => {
    globalThis.__floraRuntimeKind = undefined
    globalThis.__floraAuthorizeFeature = undefined
  })

  it('registers an async boolean authorization handler in an orchestrator runtime', async () => {
    globalThis.__floraRuntimeKind = 'orchestrator'
    defineOrchestrator({
      authorizeFeature: ({ feature, userId }) => feature === 'custom_bots' && userId === '123'
    })

    await expect(
      globalThis.__floraAuthorizeFeature!({ feature: 'custom_bots', userId: '123' })
    ).resolves.toBe(true)
    await expect(
      globalThis.__floraAuthorizeFeature!({ feature: 'custom_bots', userId: '456' })
    ).resolves.toBe(false)
  })

  it('rejects registration outside the orchestrator runtime', () => {
    expect(() => defineOrchestrator({ authorizeFeature: () => true })).toThrow(
      'trusted orchestrator deployment'
    )
  })
})
