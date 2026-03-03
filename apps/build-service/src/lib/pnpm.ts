import { execFile } from 'node:child_process'
import fs from 'node:fs/promises'
import path from 'node:path'

import { getPnpmStoreDir } from '../env'

const PNPM_INSTALL_TIMEOUT = 60_000 // 60 seconds

export async function pnpmInstall(
  workspaceDir: string,
  onLog: (line: string) => void
): Promise<void> {
  const storeDir = getPnpmStoreDir()
  await fs.mkdir(storeDir, { recursive: true })
  onLog(`Using pnpm store: ${storeDir}`)

  const hasLockfile = await fs
    .access(path.join(workspaceDir, 'pnpm-lock.yaml'))
    .then(() => true)
    .catch(() => false)

  if (!hasLockfile) {
    onLog('No lockfile found, resolving dependencies...')
    await runPnpm(
      workspaceDir,
      [
        'install',
        '--lockfile-only',
        '--ignore-scripts',
        '--ignore-workspace',
        '--store-dir',
        storeDir
      ],
      onLog
    )
  }

  onLog('Installing dependencies...')
  await runPnpm(
    workspaceDir,
    [
      'install',
      '--frozen-lockfile',
      '--ignore-scripts',
      '--ignore-workspace',
      '--store-dir',
      storeDir
    ],
    onLog
  )
}

function runPnpm(cwd: string, args: string[], onLog: (line: string) => void): Promise<void> {
  return new Promise((resolve, reject) => {
    const child = execFile('pnpm', args, {
      cwd,
      timeout: PNPM_INSTALL_TIMEOUT,
      env: {
        ...process.env,
        CI: 'true',
        npm_config_ignore_scripts: 'true'
      }
    }, (error, stdout, stderr) => {
      if (error) {
        if (stderr) onLog(stderr)
        reject(new Error(`pnpm ${args.join(' ')} failed: ${error.message}`))
        return
      }
      resolve()
    })

    child.stdout?.on('data', (data: Buffer) => {
      for (const line of data.toString().split('\n')) {
        const trimmed = line.trim()
        if (trimmed) onLog(trimmed)
      }
    })

    child.stderr?.on('data', (data: Buffer) => {
      for (const line of data.toString().split('\n')) {
        const trimmed = line.trim()
        if (trimmed) onLog(trimmed)
      }
    })
  })
}
