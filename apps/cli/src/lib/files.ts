import { readFile } from 'node:fs/promises'
import path from 'node:path'

import ignore from 'ignore'
import { glob } from 'tinyglobby'

export type DeploymentFile = {
  path: string
  contents: string
}

const ALLOWED_EXTENSIONS = new Set(['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cts'])
const EXTRA_FILE_NAMES = new Set([
  'package.json',
  'pnpm-lock.yaml',
  'package-lock.json',
  'yarn.lock',
  'bun.lockb',
  'tsconfig.json'
])
const SKIP_DIRS = new Set([
  'node_modules',
  'target',
  'dist',
  '.output',
  '.next',
  '.nuxt',
  '.svelte-kit',
  'build',
  'out',
  '.turbo',
  '.cache',
  'coverage',
  '.parcel-cache',
  '.vite',
  '.git'
])

export async function collectFiles(root: string): Promise<DeploymentFile[]> {
  const rootAbs = path.resolve(root)
  const ignorePatterns = [...SKIP_DIRS].map((dir) => `**/${dir}/**`)
  const includePatterns = [
    ...[...ALLOWED_EXTENSIONS].map((ext) => `**/*${ext}`),
    ...[...EXTRA_FILE_NAMES].map((fileName) => `**/${fileName}`)
  ]
  const relPaths = await glob(includePatterns, {
    cwd: rootAbs,
    dot: true,
    onlyFiles: true,
    ignore: ignorePatterns,
    followSymbolicLinks: false
  })

  const ignoreMatcher = await buildIgnoreMatcher(rootAbs, ignorePatterns)

  const files: DeploymentFile[] = []
  for (const rel of relPaths) {
    if (ignoreMatcher.ignores(rel)) {
      continue
    }

    const abs = path.join(rootAbs, rel)
    const contents = await readFile(abs, 'utf8')
    files.push({ path: rel.replace(/\\/g, '/'), contents })
  }

  if (files.length === 0) {
    throw new Error(`No files found under ${rootAbs}`)
  }

  return files
}

async function buildIgnoreMatcher(rootAbs: string, ignorePatterns: string[]) {
  const matcher = ignore()
  const ignoreFiles = await glob(['.gitignore', '.ignore', '**/.gitignore', '**/.ignore'], {
    cwd: rootAbs,
    dot: true,
    onlyFiles: true,
    ignore: ignorePatterns,
    followSymbolicLinks: false
  })

  ignoreFiles.sort((a, b) => depth(a) - depth(b) || a.localeCompare(b))

  for (const rel of ignoreFiles) {
    const content = await readFile(path.join(rootAbs, rel), 'utf8')
    const dir = path.posix.dirname(rel)
    const prefix = dir === '.' ? '' : `${dir}/`

    const patterns = content
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith('#'))
      .map((pattern) => toRootPattern(pattern, prefix))
      .filter((pattern): pattern is string => Boolean(pattern))

    matcher.add(patterns)
  }

  return matcher
}

function toRootPattern(rawPattern: string, prefix: string): string | null {
  const negated = rawPattern.startsWith('!')
  let pattern = negated ? rawPattern.slice(1) : rawPattern
  if (!pattern) {
    return null
  }

  const rooted = pattern.startsWith('/')
  if (rooted) {
    pattern = pattern.slice(1)
  }
  if (!pattern) {
    return null
  }

  const withTree = pattern.endsWith('/') ? `${pattern}**` : pattern
  const fullPattern = rooted || withTree.includes('/')
    ? `${prefix}${withTree}`
    : `${prefix}**/${withTree}`

  return negated ? `!${fullPattern}` : fullPattern
}

function depth(relPath: string): number {
  return relPath.split('/').length
}

export function toRelative(filePath: string, root: string): string {
  const rel = path.relative(root, filePath).replace(/\\/g, '/')

  if (!rel || rel.startsWith('..')) {
    throw new Error(`Entry file is not inside ${root}`)
  }

  return rel
}
