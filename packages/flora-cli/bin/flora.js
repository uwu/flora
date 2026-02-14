#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const isWindows = process.platform === 'win32'
const binaryPath = join(here, isWindows ? 'flora.exe' : 'flora')

if (!existsSync(binaryPath)) {
  console.error('flora binary missing. reinstall @uwu/flora-cli to run postinstall.')
  process.exit(1)
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit'
})

if (result.error) {
  console.error(`failed to execute flora binary: ${result.error.message}`)
  process.exit(1)
}

if (typeof result.status === 'number') {
  process.exit(result.status)
}

process.exit(1)
