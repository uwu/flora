import { readFile, stat } from 'node:fs/promises'
import path from 'node:path'

import { zipSync } from 'fflate'
import { glob } from 'tinyglobby'

const MAX_FILE_COUNT = 1000
const INCLUDE_PATTERNS = ['src/**']
const INCLUDE_FILES = ['package.json', 'pnpm-lock.yaml', 'flora.config.ts']

export async function zipProject(root: string): Promise<{ zip: Uint8Array; fileCount: number }> {
  const rootAbs = path.resolve(root)

  const sourceFiles = await glob(INCLUDE_PATTERNS, {
    cwd: rootAbs,
    onlyFiles: true,
    followSymbolicLinks: false,
    ignore: ['node_modules/**', '.git/**']
  })

  const topLevelFiles: string[] = []
  for (const file of INCLUDE_FILES) {
    const fullPath = path.join(rootAbs, file)
    const exists = await stat(fullPath).catch(() => null)
    if (exists?.isFile()) {
      topLevelFiles.push(file)
    }
  }

  const allFiles = [...sourceFiles, ...topLevelFiles]

  if (allFiles.length === 0) {
    throw new Error(`No files found under ${rootAbs}`)
  }

  if (allFiles.length > MAX_FILE_COUNT) {
    throw new Error(`Project has ${allFiles.length} files, exceeding limit of ${MAX_FILE_COUNT}`)
  }

  const entries: Record<string, Uint8Array> = {}
  for (const rel of allFiles) {
    const abs = path.join(rootAbs, rel)
    const contents = await readFile(abs)
    entries[rel.replace(/\\/g, '/')] = new Uint8Array(contents)
  }

  const zip = zipSync(entries)
  return { zip, fileCount: allFiles.length }
}
