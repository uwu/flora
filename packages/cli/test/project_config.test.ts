import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { describe, expect, it } from 'vite-plus/test'

import { loadProjectConfig } from '../src/lib/config'

describe('loadProjectConfig', () => {
  it('returns empty deploy config when flora.config.ts is missing', async () => {
    const root = await mkdtemp(path.join(tmpdir(), 'flora-cli-config-'))

    try {
      expect(await loadProjectConfig(root)).toEqual({})
    } finally {
      await rm(root, { recursive: true, force: true })
    }
  })

  it('loads deploy options from flora.config.ts', async () => {
    const root = await mkdtemp(path.join(tmpdir(), 'flora-cli-config-'))

    try {
      await mkdir(path.join(root, 'bot'), { recursive: true })
      await writeFile(
        path.join(root, 'flora.config.ts'),
        `export default {
  deploy: {
    entry: 'bot/main.ts',
    root: './bot',
    sourcemap: 'external',
    minify: true,
    external: ['discord.js']
  }
}\n`
      )

      expect(await loadProjectConfig(root)).toEqual({
        entry: 'bot/main.ts',
        root: './bot',
        sourcemap: 'external',
        minify: true,
        external: ['discord.js']
      })
    } finally {
      await rm(root, { recursive: true, force: true })
    }
  })
})
