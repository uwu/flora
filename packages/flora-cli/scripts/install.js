#!/usr/bin/env node

import { createHash } from 'node:crypto'
import { createWriteStream } from 'node:fs'
import { promises as fs } from 'node:fs'
import { dirname, join } from 'node:path'
import { Readable } from 'node:stream'
import { pipeline } from 'node:stream/promises'
import { fileURLToPath } from 'node:url'

const TARGET_MAP = {
  'linux:x64': { triple: 'x86_64-unknown-linux-gnu', ext: '' },
  'darwin:x64': { triple: 'x86_64-apple-darwin', ext: '' },
  'darwin:arm64': { triple: 'aarch64-apple-darwin', ext: '' },
  'win32:x64': { triple: 'x86_64-pc-windows-msvc', ext: '.exe' }
}

async function main() {
  if (process.env.FLORA_CLI_SKIP_INSTALL === '1') {
    console.log('Skipping flora binary download (FLORA_CLI_SKIP_INSTALL=1)')
    return
  }

  const target = TARGET_MAP[`${process.platform}:${process.arch}`]
  if (!target) {
    throw new Error(`unsupported platform/arch: ${process.platform}/${process.arch}`)
  }

  const packageRoot = join(dirname(fileURLToPath(import.meta.url)), '..')
  const packageJsonPath = join(packageRoot, 'package.json')
  const packageJson = JSON.parse(await fs.readFile(packageJsonPath, 'utf8'))
  const version = packageJson.version

  if (typeof version !== 'string' || version.length === 0) {
    throw new Error('package version missing')
  }
  if (version === '0.0.0') {
    console.log('Skipping flora binary download for placeholder version 0.0.0')
    return
  }

  const releaseTag = `flora-cli-v${version}`
  const assetName = `flora-${target.triple}${target.ext}`
  const releaseBaseUrl = `https://github.com/uwu/flora/releases/download/${releaseTag}`
  const assetUrl = `${releaseBaseUrl}/${assetName}`
  const checksumsUrl = `${releaseBaseUrl}/checksums.txt`

  const tempDir = join(packageRoot, '.tmp-install')
  const tempBinaryPath = join(tempDir, assetName)
  const finalBinaryPath = join(
    packageRoot,
    'bin',
    process.platform === 'win32' ? 'flora.exe' : 'flora'
  )

  await fs.rm(tempDir, { recursive: true, force: true })
  await fs.mkdir(tempDir, { recursive: true })

  await downloadToFile(assetUrl, tempBinaryPath)

  const checksums = await downloadText(checksumsUrl)
  const expected = readExpectedChecksum(checksums, assetName)
  if (!expected) {
    throw new Error(`checksum entry missing for ${assetName}`)
  }

  const actual = await sha256File(tempBinaryPath)
  if (actual !== expected) {
    throw new Error(`checksum mismatch for ${assetName}`)
  }

  await fs.mkdir(dirname(finalBinaryPath), { recursive: true })
  await fs.copyFile(tempBinaryPath, finalBinaryPath)

  if (process.platform !== 'win32') {
    await fs.chmod(finalBinaryPath, 0o755)
  }

  await fs.rm(tempDir, { recursive: true, force: true })
  console.log(`Installed flora (${target.triple})`)
}

async function downloadToFile(url, filePath) {
  const response = await fetch(url, { redirect: 'follow' })
  if (!response.ok || !response.body) {
    throw new Error(`download failed: ${url} (${response.status})`)
  }

  await fs.mkdir(dirname(filePath), { recursive: true })
  await pipeline(Readable.fromWeb(response.body), createWriteStream(filePath))
}

async function downloadText(url) {
  const response = await fetch(url, { redirect: 'follow' })
  if (!response.ok) {
    throw new Error(`download failed: ${url} (${response.status})`)
  }
  return response.text()
}

function readExpectedChecksum(contents, assetName) {
  const lines = contents.split('\n')

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed) {
      continue
    }

    const match = /^([a-fA-F0-9]{64})\s+\*?(.+)$/.exec(trimmed)
    if (!match) {
      continue
    }

    const [, checksum, fileName] = match
    if (fileName === assetName || fileName.endsWith(`/${assetName}`)) {
      return checksum.toLowerCase()
    }
  }

  return null
}

async function sha256File(filePath) {
  const content = await fs.readFile(filePath)
  return createHash('sha256').update(content).digest('hex')
}

main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error)
  console.error(`failed to install flora binary: ${message}`)
  process.exit(1)
})
