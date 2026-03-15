import path from 'node:path'

import { build } from 'rolldown'
import type { OutputAsset, OutputChunk } from 'rolldown'

import { isBundleMinifyEnabled } from '../env'

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
  const minifyEnabled = isBundleMinifyEnabled()
  const minifyOption = minifyEnabled
    ? {
        compress: {
          keepNames: {
            function: true,
            class: true
          }
        },
        mangle: {
          keepNames: true
        }
      }
    : false

  const result = await build({
    cwd: workspaceDir,
    input: entryAbs,
    write: false,
    output: {
      format: 'esm',
      sourcemap: true,
      exports: 'named',
      minify: minifyOption,
      keepNames: minifyEnabled
    },
    checks: {
      eval: false
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
