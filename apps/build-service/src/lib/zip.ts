import fs from 'node:fs/promises'
import path from 'node:path'

import { unzipSync } from 'fflate'

export const MAX_COMPRESSED_SIZE = 50 * 1024 * 1024 // 50 MB
const MAX_FILE_COUNT = 1000
const MAX_INDIVIDUAL_FILE_SIZE = 10 * 1024 * 1024 // 10 MB
const MAX_TOTAL_UNCOMPRESSED_SIZE = 200 * 1024 * 1024 // 200 MB

export type ExtractResult = {
  fileCount: number
  totalSize: number
}

export async function extractZip(zipData: Uint8Array, destDir: string): Promise<ExtractResult> {
  if (zipData.byteLength > MAX_COMPRESSED_SIZE) {
    throw new Error(`Zip file exceeds maximum compressed size of ${MAX_COMPRESSED_SIZE} bytes`)
  }

  let scannedFileCount = 0
  let scannedTotalSize = 0
  const entries = unzipSync(zipData, {
    filter(file) {
      validatePath(file.name)

      if (file.name.endsWith('/')) return false

      scannedFileCount += 1
      if (scannedFileCount > MAX_FILE_COUNT) {
        throw new Error(`Zip contains more than ${MAX_FILE_COUNT} files`)
      }

      if (file.originalSize > MAX_INDIVIDUAL_FILE_SIZE) {
        throw new Error(
          `File ${file.name} exceeds maximum size of ${MAX_INDIVIDUAL_FILE_SIZE} bytes`
        )
      }

      scannedTotalSize += file.originalSize
      if (scannedTotalSize > MAX_TOTAL_UNCOMPRESSED_SIZE) {
        throw new Error(
          `Total uncompressed size exceeds limit of ${MAX_TOTAL_UNCOMPRESSED_SIZE} bytes`
        )
      }

      return true
    }
  })
  const entryNames = Object.keys(entries)

  if (entryNames.length > MAX_FILE_COUNT) {
    throw new Error(`Zip contains ${entryNames.length} files, exceeding limit of ${MAX_FILE_COUNT}`)
  }

  let totalSize = 0

  for (const name of entryNames) {
    validatePath(name)

    const data = entries[name]!
    if (data.byteLength > MAX_INDIVIDUAL_FILE_SIZE) {
      throw new Error(`File ${name} exceeds maximum size of ${MAX_INDIVIDUAL_FILE_SIZE} bytes`)
    }

    totalSize += data.byteLength
    if (totalSize > MAX_TOTAL_UNCOMPRESSED_SIZE) {
      throw new Error(
        `Total uncompressed size exceeds limit of ${MAX_TOTAL_UNCOMPRESSED_SIZE} bytes`
      )
    }

    // skip directories (entries ending with /)
    if (name.endsWith('/')) continue

    const destPath = path.join(destDir, name)
    const resolved = path.resolve(destPath)
    const root = path.resolve(destDir)
    if (resolved !== root && !resolved.startsWith(`${root}${path.sep}`)) {
      throw new Error(`Path traversal detected: ${name}`)
    }

    await fs.mkdir(path.dirname(destPath), { recursive: true })
    await fs.writeFile(destPath, data)
  }

  return {
    fileCount: entryNames.filter((n) => !n.endsWith('/')).length,
    totalSize
  }
}

function validatePath(name: string): void {
  if (name.includes('\0')) {
    throw new Error(`Null byte not allowed in path: ${name}`)
  }

  if (path.isAbsolute(name) || path.win32.isAbsolute(name)) {
    throw new Error(`Absolute path not allowed: ${name}`)
  }

  const segments = name.split(/[\\/]+/)
  for (const segment of segments) {
    if (segment === '..') {
      throw new Error(`Path traversal not allowed: ${name}`)
    }
  }
}
