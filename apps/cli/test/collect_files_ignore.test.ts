import assert from 'node:assert/strict'
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { describe, it } from 'node:test'

import { collectFiles } from '../src/lib/files'

describe('collectFiles ignore rules', () => {
  it('respects .gitignore and .ignore files', async () => {
    const root = await mkdtemp(path.join(tmpdir(), 'flora-cli-files-'))

    try {
      await mkdir(path.join(root, 'src'), { recursive: true })
      await writeFile(path.join(root, '.gitignore'), 'ignored.ts\n')
      await writeFile(path.join(root, '.ignore'), 'hidden.js\n')
      await writeFile(path.join(root, 'src', 'main.ts'), 'export const ok = true\n')
      await writeFile(path.join(root, 'ignored.ts'), 'export const bad = true\n')
      await writeFile(path.join(root, 'hidden.js'), 'module.exports = 1\n')

      const files = await collectFiles(root)
      const paths = files.map((file) => file.path).sort()

      assert.deepEqual(paths, ['src/main.ts'])
    } finally {
      await rm(root, { recursive: true, force: true })
    }
  })
})
