import fs from 'node:fs/promises'
import path from 'node:path'

import { getBuildRunsDir } from '../env'
import { appendLog, setBuildArtifact, setBuildError, updateBuildStatus } from './builds'
import { bundleProject } from './bundle'
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
  const runsDir = getBuildRunsDir()
  await fs.mkdir(runsDir, { recursive: true })
  const tmpDir = await fs.mkdtemp(path.join(runsDir, `build-${buildId}-`))

  const log = (line: string) => appendLog(buildId, line)

  const timeout = setTimeout(() => {
    setBuildError(buildId, 'Build timed out')
    updateBuildStatus(buildId, 'failed')
  }, BUILD_TIMEOUT)

  try {
    updateBuildStatus(buildId, 'running')

    // 1. extract zip
    log('Extracting project...')
    const extracted = await extractZip(zipData, tmpDir)
    log(`Extracted ${extracted.fileCount} files (${formatBytes(extracted.totalSize)})`)

    // 2. validate & sanitize package.json
    log('Validating package.json...')
    const pkg = await validateAndSanitizePackageJson(tmpDir)
    const depCount = pkg.dependencies ? Object.keys(pkg.dependencies).length : 0
    log(`Found ${depCount} dependencies`)

    // 3. install dependencies (only if there are any)
    if (depCount > 0) {
      await pnpmInstall(tmpDir, log)
    }

    // 4. bundle with rolldown
    const result = await bundleProject(tmpDir, entry, log)

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
    const message = err instanceof Error ? err.message : String(err)
    log(`Build failed: ${message}`)
    setBuildError(buildId, message)
    updateBuildStatus(buildId, 'failed')
  } finally {
    clearTimeout(timeout)
    // clean up temp directory
    await fs.rm(tmpDir, { recursive: true, force: true }).catch(() => {})
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}kb`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}
