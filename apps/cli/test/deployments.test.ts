import { beforeEach, describe, expect, it, vi } from 'vitest'

import { deploy } from '../src/commands/deployments'
import type { CliConfig } from '../src/lib/types'

const {
  postMock,
  expectOkMock,
  loadProjectConfigMock,
  bundleDeploymentSourceMock
} = vi.hoisted(() => ({
  postMock: vi.fn(),
  expectOkMock: vi.fn(),
  loadProjectConfigMock: vi.fn(),
  bundleDeploymentSourceMock: vi.fn()
}))

vi.mock('../src/lib/http', () => ({
  authHeaders: vi.fn(() => ({ authorization: 'Bearer token' })),
  createApiClient: vi.fn(() => ({
    POST: postMock
  })),
  expectOk: expectOkMock
}))

vi.mock('../src/lib/config', () => ({
  loadProjectConfig: loadProjectConfigMock
}))

vi.mock('../src/lib/deploy_bundle', () => ({
  bundleDeploymentSource: bundleDeploymentSourceMock
}))

vi.mock('../src/lib/logger', () => ({
  logger: {
    log: vi.fn()
  }
}))

describe('deploy command', () => {
  const config: CliConfig = {
    apiUrl: 'http://localhost:3000/api',
    token: 'token'
  }

  beforeEach(() => {
    postMock.mockReset()
    expectOkMock.mockReset()
    loadProjectConfigMock.mockReset()
    bundleDeploymentSourceMock.mockReset()

    loadProjectConfigMock.mockResolvedValue({})
    bundleDeploymentSourceMock.mockResolvedValue({
      entry: 'src/main.ts',
      bundle: 'console.log(1)',
      sourceMap: {
        path: 'main.js.map',
        contents: '{"version":3}'
      }
    })
    expectOkMock.mockResolvedValue({
      guild_id: '123',
      entry: 'src/main.ts',
      updated_at: 'now'
    })
    postMock.mockResolvedValue({})
  })

  it('posts prebundled deployment payload', async () => {
    await deploy(config, '123', undefined)

    expect(postMock).toHaveBeenCalledOnce()
    const request = postMock.mock.calls[0]?.[1]

    expect(request).toMatchObject({
      body: {
        entry: 'src/main.ts',
        bundle: 'console.log(1)',
        source_map: {
          path: 'main.js.map',
          contents: '{"version":3}'
        }
      }
    })
  })

  it('merges cli overrides over project config', async () => {
    loadProjectConfigMock.mockResolvedValue({
      entry: 'config/entry.ts',
      root: './bot',
      sourcemap: 'external',
      minify: false,
      external: ['discord.js']
    })

    await deploy(config, '123', 'src/cli-entry.ts', {
      root: './cli-root',
      sourcemap: 'inline',
      minify: true,
      external: ['node:fs']
    })

    expect(bundleDeploymentSourceMock).toHaveBeenCalledWith({
      root: './cli-root',
      entry: 'src/cli-entry.ts',
      sourcemap: 'inline',
      minify: true,
      external: ['node:fs']
    })
  })
})
