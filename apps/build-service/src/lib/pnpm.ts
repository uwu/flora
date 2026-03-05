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
        '--color=always',
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
      '--color=always',
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
        FORCE_COLOR: '1',
        npm_config_color: 'always',
        npm_config_ignore_scripts: 'true'
      }
    }, (error, stdout, stderr) => {
      if (error) {
        emitBufferedLines(stderr, onLog)
        reject(new Error(`pnpm ${args.join(' ')} failed: ${error.message}`))
        return
      }
      resolve()
    })

    streamLines(child.stdout, onLog)
    streamLines(child.stderr, onLog)
  })
}

function streamLines(
  stream: NodeJS.ReadableStream | null | undefined,
  onLog: (line: string) => void
): void {
  if (!stream) return

  let pending = ''
  stream.on('data', (chunk: Buffer) => {
    pending += chunk.toString()
    let newlineIndex = pending.indexOf('\n')
    while (newlineIndex !== -1) {
      const line = pending.slice(0, newlineIndex).replace(/\r$/, '')
      if (line.length > 0) onLog(line)
      pending = pending.slice(newlineIndex + 1)
      newlineIndex = pending.indexOf('\n')
    }
  })

  stream.on('end', () => {
    const line = pending.replace(/\r$/, '')
    if (line.length > 0) onLog(line)
  })
}

function emitBufferedLines(output: string, onLog: (line: string) => void): void {
  for (const line of output.split('\n')) {
    const next = line.replace(/\r$/, '')
    if (next.length > 0) onLog(next)
  }
}
