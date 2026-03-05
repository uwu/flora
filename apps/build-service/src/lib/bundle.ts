import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { build } from 'rolldown'
import type { OutputAsset, OutputChunk } from 'rolldown'
import { defineEnv } from 'unenv'

import { packageShimPlugin } from './third-party/shim-package'

const { env } = defineEnv({
  nodeCompat: true,
  npmShims: true,
  resolve: true,
  overrides: {}
})

const thisDir = path.dirname(fileURLToPath(import.meta.url))
const polyfillsDir = path.resolve(thisDir, 'third-party/polyfills')

const BROWSER_ALIASES: Record<string, string> = {
  ...env.alias,
  // CJS modules required due to ESM import heuristics
  events: path.join(polyfillsDir, 'eventemitter-polyfill.cjs'),
  'node:events': path.join(polyfillsDir, 'eventemitter-polyfill.cjs'),
  timers: path.join(polyfillsDir, 'timer-polyfill.ts'),
  'node:timers': path.join(polyfillsDir, 'timer-polyfill.ts'),
  'timers/promises': path.join(polyfillsDir, 'timer-promises-polyfill.ts'),
  'node:timers/promises': path.join(polyfillsDir, 'timer-promises-polyfill.ts'),
  'zlib-sync': path.join(polyfillsDir, 'zlib-sync-polyfill.ts')
}

export type BundleResult = {
  bundle: string
  sourceMap: string
}

export async function bundleProject(
  workspaceDir: string,
  entry: string,
  onLog: (line: string) => void
): Promise<BundleResult> {
  const entryAbs = path.resolve(workspaceDir, entry)
  const entryRel = path.relative(workspaceDir, entryAbs).replace(/\\/g, '/')

  if (!entryRel || entryRel.startsWith('..')) {
    throw new Error(`Entry file is not inside workspace: ${entry}`)
  }

  onLog(`Bundling ${entryRel}`)

  const result = await build({
    cwd: workspaceDir,
    input: entryAbs,
    write: false,
    platform: 'browser',
    plugins: [
      packageShimPlugin({
        package: 'ws',
        path: path.join(polyfillsDir, 'ws-polyfill.ts')
      })
    ],
    resolve: {
      alias: BROWSER_ALIASES
    },
    transform: {
      inject: {
        ...(env.inject as Record<string, string | [string, string]>)
      }
    },
    external: [],
    output: {
      format: 'iife',
      sourcemap: true,
      codeSplitting: false
    }
  })

  const chunk = result.output.find(
    (output): output is OutputChunk => output.type === 'chunk' && output.isEntry
  )

  if (!chunk) {
    throw new Error(`Failed to bundle entry ${entryRel}`)
  }

  const sourceMapAsset = result.output.find(
    (output): output is OutputAsset => output.type === 'asset' && output.fileName.endsWith('.map')
  )

  const bundleSize = Buffer.byteLength(chunk.code, 'utf-8')
  onLog(`Bundle size: ${formatBytes(bundleSize)}`)

  return {
    bundle: chunk.code,
    sourceMap: sourceMapAsset ? toText(sourceMapAsset.source) : '{}'
  }
}

function toText(value: string | Uint8Array): string {
  if (typeof value === 'string') return value
  return Buffer.from(value).toString('utf8')
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}kb`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}
