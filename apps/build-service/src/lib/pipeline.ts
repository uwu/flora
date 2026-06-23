import fs from 'node:fs/promises'
import path from 'node:path'

import { colors, stripAnsi } from 'consola/utils'

import { getBuildRunsDir } from '../env'
import { appendLog, setBuildArtifact, setBuildError, updateBuildStatus } from './builds'
import { bundleProject } from './bundle'
import { logger } from './logger'
import { pnpmInstall } from './pnpm'
import { validateAndSanitizePackageJson } from './validate-package'
import { extractZip } from './zip'

const BUILD_TIMEOUT = 120_000 // 120 seconds
const MAX_BUNDLE_SIZE = 25 * 1024 * 1024 // 25 MB raw

export async function runBuildPipeline(
  buildId: string,
  guildId: string,
  entry: string,
  zipData: Uint8Array
): Promise<void> {
  const buildLogger = logger.withTag(colors.magenta(`build:${buildId.slice(0, 8)}`))
  const controller = new AbortController()
  const timeout = setTimeout(() => {
    controller.abort(new Error('Build timed out'))
  }, BUILD_TIMEOUT)
  let tmpDir: string | undefined

  const log = (line: string) => {
    appendLog(buildId, stripAnsi(line))
    if (line.includes('\x1b[')) {
      buildLogger.log(line)
      return
    }
    buildLogger.info(line)
  }

  try {
    const runsDir = getBuildRunsDir()
    await fs.mkdir(runsDir, { recursive: true })
    tmpDir = await fs.mkdtemp(path.join(runsDir, `build-${buildId}-`))

    updateBuildStatus(buildId, 'running')

    // 1. extract zip
    log('Extracting project...')
    const extracted = await abortable(extractZip(zipData, tmpDir), controller.signal)
    log(`Extracted ${extracted.fileCount} files (${formatBytes(extracted.totalSize)})`)
    throwIfAborted(controller.signal)

    // 2. validate & sanitize package.json
    log('Validating package.json...')
    const pkg = await validateAndSanitizePackageJson(tmpDir)
    const depCount = pkg.dependencies ? Object.keys(pkg.dependencies).length : 0
    log(`Found ${depCount} dependencies`)
    throwIfAborted(controller.signal)

    // 3. install dependencies (only if there are any)
    if (depCount > 0) {
      await pnpmInstall(tmpDir, log, controller.signal)
    }
    throwIfAborted(controller.signal)

    // 4. bundle with rolldown
    const result = await abortable(bundleProject(tmpDir, entry, log), controller.signal)
    throwIfAborted(controller.signal)

    // 5. check bundle size limits
    const rawSize = Buffer.byteLength(result.bundle, 'utf-8')
    if (rawSize > MAX_BUNDLE_SIZE) {
      throw new Error(
        `Bundle size ${formatBytes(rawSize)} exceeds limit of ${formatBytes(MAX_BUNDLE_SIZE)}`
      )
    }

    log('Sourcemap generated')

    // 6. store artifact
    setBuildArtifact(buildId, {
      bundle: result.bundle,
      sourceMap: result.sourceMap
    })

    updateBuildStatus(buildId, 'done')
    log('Build complete')
  } catch (err) {
    const message = controller.signal.aborted ? 'Build timed out' : errToMessage(err)
    log(`Build failed: ${message}`)
    setBuildError(buildId, message)
    updateBuildStatus(buildId, 'failed')
  } finally {
    clearTimeout(timeout)
    // clean up temp directory
    if (tmpDir) {
      await fs.rm(tmpDir, { recursive: true, force: true }).catch(() => {})
    }
  }
}

function abortable<T>(promise: Promise<T>, signal: AbortSignal): Promise<T> {
  if (signal.aborted) {
    return Promise.reject(abortReason(signal))
  }

  return new Promise((resolve, reject) => {
    const onAbort = () => reject(abortReason(signal))
    signal.addEventListener('abort', onAbort, { once: true })

    promise.then(resolve, reject).finally(() => {
      signal.removeEventListener('abort', onAbort)
    })
  })
}

function throwIfAborted(signal: AbortSignal): void {
  if (signal.aborted) {
    throw abortReason(signal)
  }
}

function abortReason(signal: AbortSignal): Error {
  return signal.reason instanceof Error ? signal.reason : new Error('Build timed out')
}

function errToMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err)
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}kb`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}
