import { zipSync } from 'fflate'

import type { DeploymentFileRecord, FileTreeNode } from './types'

const BUILD_LOCKFILES = new Set(['pnpm-lock.yaml', 'package-lock.json', 'yarn.lock', 'bun.lockb'])
const BUILD_TOP_LEVEL_FILES = new Set(['package.json', 'flora.config.ts'])

export function formatLogLine(entry: {
  timestamp: number
  level: string
  target: string
  message: string
}) {
  const time = new Date(entry.timestamp).toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
  return `[${time}] ${entry.level} ${entry.target}: ${entry.message}`
}

export const floraSdkModuleTypes = `
declare module '@uwu/flora-sdk' {
  export const on: typeof globalThis.on
  export const cron: typeof globalThis.cron
  export const secrets: typeof globalThis.secrets
  export const prefix: typeof globalThis.prefix
  export const slash: typeof globalThis.slash
  export const createBot: typeof globalThis.createBot
  export const embed: typeof globalThis.embed
  export const ActionRowBuilder: typeof globalThis.ActionRowBuilder
  export const ButtonBuilder: typeof globalThis.ButtonBuilder
  export const StringSelectMenuBuilder: typeof globalThis.StringSelectMenuBuilder
}
`

export function toMonacoUri(path: string) {
  return `file:///${path.replace(/^\/+/, '')}`
}

export function getLanguageFromPath(path: string): string {
  if (path.endsWith('.json')) return 'json'
  if (
    path.endsWith('.ts') || path.endsWith('.tsx') || path.endsWith('.cts') || path.endsWith('.mts')
  ) {
    return 'typescript'
  }
  if (
    path.endsWith('.js') || path.endsWith('.jsx') || path.endsWith('.cjs') || path.endsWith('.mjs')
  ) {
    return 'javascript'
  }
  return 'plaintext'
}

function sortNodes(nodes: FileTreeNode[]): FileTreeNode[] {
  return nodes
    .sort((a, b) => {
      if (a.kind !== b.kind) return a.kind === 'folder' ? -1 : 1
      return a.name.localeCompare(b.name)
    })
    .map((node) => {
      if (node.kind === 'folder' && node.children) {
        return { ...node, children: sortNodes(node.children) }
      }
      return node
    })
}

export function normalizePath(input: string) {
  return input.trim().replace(/^\/+/, '').replace(/\/+/g, '/').replace(/\/$/, '')
}

export function getParentFolder(path: string) {
  const normalized = normalizePath(path)
  const idx = normalized.lastIndexOf('/')
  if (idx === -1) return ''
  return normalized.slice(0, idx)
}

export function collectParentFolders(path: string): string[] {
  const normalized = normalizePath(path)
  const parts = normalized.split('/').filter(Boolean)
  const folders: string[] = []
  let current = ''
  for (let i = 0; i < parts.length - 1; i += 1) {
    current = current ? `${current}/${parts[i]}` : parts[i]
    folders.push(current)
  }
  return folders
}

export function buildFileTree(filePaths: string[], explicitFolders: string[]): FileTreeNode[] {
  const root: FileTreeNode[] = []

  const insertPath = (fullPath: string, forceFolderLeaf: boolean) => {
    const parts = fullPath.split('/').filter(Boolean)
    if (parts.length === 0) return

    let level = root
    let currentPath = ''

    for (let i = 0; i < parts.length; i += 1) {
      const part = parts[i]
      currentPath = currentPath ? `${currentPath}/${part}` : part
      const isLast = i === parts.length - 1
      const isFolderLeaf = forceFolderLeaf && isLast
      const kind: FileTreeNode['kind'] = isFolderLeaf ? 'folder' : isLast ? 'file' : 'folder'

      let node = level.find((candidate) => candidate.path === currentPath)
      if (!node) {
        node = { kind, name: part, path: currentPath, children: kind === 'folder' ? [] : undefined }
        level.push(node)
      }

      if (node.kind === 'folder') {
        if (!node.children) node.children = []
        level = node.children
      }
    }
  }

  for (const folderPath of explicitFolders) {
    insertPath(folderPath, true)
  }
  for (const filePath of filePaths) {
    insertPath(filePath, false)
  }

  return sortNodes(root)
}

export function createZipFromFiles(files: DeploymentFileRecord) {
  const textEncoder = new TextEncoder()
  const entries: Record<string, Uint8Array> = {}
  let hasPackageJson = false

  for (const [path, contents] of Object.entries(files)) {
    const normalized = normalizePath(path)
    if (!normalized) continue
    const fileName = normalized.split('/').pop() ?? normalized
    if (BUILD_LOCKFILES.has(fileName)) continue
    const shouldIncludeInBuild = normalized.startsWith('src/') ||
      BUILD_TOP_LEVEL_FILES.has(normalized)
    if (!shouldIncludeInBuild) continue
    if (normalized === 'package.json') hasPackageJson = true
    entries[normalized] = textEncoder.encode(contents)
  }

  if (!hasPackageJson) {
    entries['package.json'] = textEncoder.encode(
      JSON.stringify(
        {
          name: '@flora/editor-deploy',
          private: true,
          type: 'module',
          dependencies: {}
        },
        null,
        2
      )
    )
  }

  const zip = zipSync(entries)
  const fileNames = Object.keys(entries).sort((a, b) => a.localeCompare(b))
  return {
    zip,
    fileCount: fileNames.length,
    fileNames
  }
}

export function parseSourceMapFromBundle(
  bundle: string
): { sources?: string[]; sourcesContent?: (string | null)[] } | null {
  const marker = 'sourceMappingURL=data:application/json'
  const markerIndex = bundle.lastIndexOf(marker)
  if (markerIndex === -1) return null

  const commaIndex = bundle.indexOf(',', markerIndex)
  if (commaIndex === -1) return null

  const endIndex = bundle.indexOf('\n', commaIndex)
  const encoded =
    (endIndex === -1 ? bundle.slice(commaIndex + 1) : bundle.slice(commaIndex + 1, endIndex)).trim()
  if (!encoded) return null

  try {
    const decoded = atob(encoded)
    return JSON.parse(decoded) as { sources?: string[]; sourcesContent?: (string | null)[] }
  } catch {
    return null
  }
}

type DeploymentLike = {
  entry?: string | null
  bundle?: string | null
  source_map?: {
    path: string
    contents: string
  } | null
  files?: Array<{ path: string; contents: string }> | null
}

export function extractFilesFromDeployment(deployment: DeploymentLike | null | undefined) {
  if (!deployment) return {}

  const files: DeploymentFileRecord = {}
  let hasSourceFiles = false

  const addSourcesFromMap = (
    sourceMap: { sources?: string[]; sourcesContent?: (string | null)[] } | null
  ) => {
    if (!sourceMap || !Array.isArray(sourceMap.sources)) return
    sourceMap.sources.forEach((sourcePath, index) => {
      const sourceContents = sourceMap.sourcesContent?.[index]
      if (typeof sourcePath === 'string' && typeof sourceContents === 'string') {
        files[sourcePath] = sourceContents
        hasSourceFiles = true
      }
    })
  }

  if (Array.isArray(deployment.files)) {
    for (const file of deployment.files) {
      if (!file.path || typeof file.path !== 'string') continue
      if (typeof file.contents !== 'string') continue
      files[file.path] = file.contents
      hasSourceFiles = true
    }
  }

  addSourcesFromMap(parseSourceMapFromBundle(deployment.bundle ?? ''))

  if (deployment.source_map?.contents && !hasSourceFiles) {
    try {
      const sourceMap = JSON.parse(deployment.source_map.contents) as {
        sources?: string[]
        sourcesContent?: (string | null)[]
      }
      addSourcesFromMap(sourceMap)
      if (!hasSourceFiles && deployment.source_map.path) {
        files[deployment.source_map.path] = deployment.source_map.contents
      }
    } catch {
      if (!hasSourceFiles && deployment.source_map.path) {
        files[deployment.source_map.path] = deployment.source_map.contents
      }
    }
  }

  if (!hasSourceFiles && deployment.bundle) {
    files[deployment.entry || 'dist/bundle.js'] = deployment.bundle
  }

  if (Object.keys(files).length === 0) {
    files['src/main.ts'] = '// No deployment bundle found for this guild yet.'
  }

  return files
}
