import { beforeEach, describe, expect, it, vi } from 'vite-plus/test'

import { deploy } from '../src/commands/deployments'
import type { CliConfig } from '../src/lib/types'

const { collectFilesMock, loadProjectConfigMock, zipProjectMock, fetchMock } = vi.hoisted(() => ({
  collectFilesMock: vi.fn(),
  loadProjectConfigMock: vi.fn(),
  zipProjectMock: vi.fn(),
  fetchMock: vi.fn()
}))

vi.mock('../src/lib/config', () => ({
  loadProjectConfig: loadProjectConfigMock
}))

vi.mock('../src/lib/zip', () => ({
  zipProject: zipProjectMock
}))

vi.mock('../src/lib/files', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../src/lib/files')>()

  return {
    ...actual,
    collectFiles: collectFilesMock
  }
})

vi.mock('../src/lib/logger', () => ({
  logger: {
    info: vi.fn(),
    log: vi.fn(),
    success: vi.fn(),
    warn: vi.fn()
  }
}))

vi.stubGlobal('fetch', fetchMock)

describe('deploy command', () => {
  const config: CliConfig = {
    apiUrl: 'http://localhost:3000/api',
    token: 'token'
  }

  beforeEach(() => {
    collectFilesMock.mockReset()
    loadProjectConfigMock.mockReset()
    zipProjectMock.mockReset()
    fetchMock.mockReset()

    collectFilesMock.mockResolvedValue([{ path: 'src/main.ts', contents: 'export default {}' }])
    loadProjectConfigMock.mockResolvedValue({})
    zipProjectMock.mockResolvedValue({
      zip: new Uint8Array([1, 2, 3]),
      fileCount: 5
    })
  })

  it('zips project and posts to /builds', async () => {
    fetchMock
      .mockResolvedValueOnce(new Response(JSON.stringify({ build_id: 'b1', status: 'queued' })))
      .mockResolvedValueOnce(new Response('event: done\ndata: ok\n\n'))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            status: 'done',
            guild_id: '123',
            entry: 'src/main.ts',
            artifact: {
              bundle: 'console.log("ok")',
              source_map: ''
            }
          })
        )
      )
      .mockResolvedValueOnce(new Response(null, { status: 204 }))

    await deploy(config, '123', undefined)

    expect(zipProjectMock).toHaveBeenCalledWith('.')
    expect(fetchMock).toHaveBeenCalledTimes(4)

    const [url, opts] = fetchMock.mock.calls[0]!
    expect(url).toBe('http://localhost:3000/api/builds')
    expect(opts.method).toBe('POST')
    expect(opts.body).toBeInstanceOf(FormData)
  })

  it('uses entry from cli arg over project config', async () => {
    loadProjectConfigMock.mockResolvedValue({ entry: 'config/entry.ts' })

    fetchMock
      .mockResolvedValueOnce(new Response(JSON.stringify({ build_id: 'b2', status: 'queued' })))
      .mockResolvedValueOnce(new Response('event: done\ndata: ok\n\n'))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            status: 'done',
            guild_id: '123',
            entry: 'src/cli-entry.ts',
            artifact: {
              bundle: 'console.log("ok")',
              source_map: ''
            }
          })
        )
      )
      .mockResolvedValueOnce(new Response(null, { status: 204 }))

    await deploy(config, '123', 'src/cli-entry.ts')

    const [, opts] = fetchMock.mock.calls[0]!
    const formData = opts.body as FormData
    expect(formData.get('entry')).toBe('src/cli-entry.ts')
  })

  it('uses root from arg over project config', async () => {
    loadProjectConfigMock.mockResolvedValue({ root: './bot' })

    fetchMock
      .mockResolvedValueOnce(new Response(JSON.stringify({ build_id: 'b3', status: 'queued' })))
      .mockResolvedValueOnce(new Response('event: done\ndata: ok\n\n'))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            status: 'done',
            guild_id: '123',
            entry: 'src/main.ts',
            artifact: {
              bundle: 'console.log("ok")',
              source_map: ''
            }
          })
        )
      )
      .mockResolvedValueOnce(new Response(null, { status: 204 }))

    await deploy(config, '123', undefined, './cli-root')

    expect(zipProjectMock).toHaveBeenCalledWith('./cli-root')
  })
})
