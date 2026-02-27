import path from 'node:path'

import { build } from 'rolldown'
import type { OutputAsset, OutputChunk } from 'rolldown'

import type { DeploySourceMapMode } from './types'

export const DEFAULT_DEPLOY_ENTRY = 'src/main.ts'
export const DEFAULT_DEPLOY_SOURCEMAP: DeploySourceMapMode = 'none'

export type DeploymentSourceMapFile = {
  path: string
  contents: string
}

export type BundleDeployOptions = {
  cwd?: string
  root?: string
  entry?: string
  sourcemap?: DeploySourceMapMode
  minify?: boolean
  external?: string[]
}

export type BundleDeployResult = {
  entry: string
  bundle: string
  sourceMap?: DeploymentSourceMapFile
}

export async function bundleDeploymentSource(
  options: BundleDeployOptions = {}
): Promise<BundleDeployResult> {
  const cwd = path.resolve(options.cwd ?? process.cwd())
  const root = path.resolve(cwd, options.root ?? '.')
  const entryAbs = path.resolve(root, options.entry ?? DEFAULT_DEPLOY_ENTRY)
  const entry = toRelativePath(entryAbs, root)

  const sourcemap = options.sourcemap ?? DEFAULT_DEPLOY_SOURCEMAP
  const minify = options.minify ?? false

  const result = await build({
    cwd: root,
    input: entryAbs,
    external: options.external,
    write: false,
    output: {
      format: 'esm',
      sourcemap: sourcemap === 'none' ? false : sourcemap === 'inline' ? 'inline' : true,
      minify,
      exports: 'named'
    }
  })

  const chunk = result.output.find(
    (output): output is OutputChunk => output.type === 'chunk' && output.isEntry
  )

  if (!chunk) {
    throw new Error(`Failed to bundle entry ${entry}`)
  }

  if (sourcemap !== 'external') {
    return {
      entry,
      bundle: chunk.code
    }
  }

  const sourceMapAsset = result.output.find(
    (output): output is OutputAsset => output.type === 'asset' && output.fileName.endsWith('.map')
  )

  if (!sourceMapAsset) {
    throw new Error(`Expected external sourcemap for entry ${entry}`)
  }

  return {
    entry,
    bundle: chunk.code,
    sourceMap: {
      path: sourceMapAsset.fileName,
      contents: toText(sourceMapAsset.source)
    }
  }
}

function toRelativePath(filePath: string, root: string): string {
  const rel = path.relative(root, filePath).replace(/\\/g, '/')

  if (!rel || rel.startsWith('..')) {
    throw new Error(`Entry file is not inside ${root}`)
  }

  return rel
}

function toText(value: string | Uint8Array): string {
  if (typeof value === 'string') {
    return value
  }

  return Buffer.from(value).toString('utf8')
}
