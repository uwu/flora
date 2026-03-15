import '@codingame/monaco-vscode-javascript-default-extension'
import '@codingame/monaco-vscode-json-default-extension'
import '@codingame/monaco-vscode-theme-defaults-default-extension'
import '@codingame/monaco-vscode-typescript-basics-default-extension'
import 'vscode/localExtensionHost'

import type { IWorkbenchConstructionOptions } from '@codingame/monaco-vscode-api'
import { initialize, LogLevel } from '@codingame/monaco-vscode-api'
import { ExtensionHostKind, registerExtension } from '@codingame/monaco-vscode-api/extensions'
import type { IDisposable } from '@codingame/monaco-vscode-api/vscode/vs/base/common/lifecycle'
import type { IFileWriteOptions } from '@codingame/monaco-vscode-api/vscode/vs/platform/files/common/files'
import getConfigurationServiceOverride, {
  initUserConfiguration
} from '@codingame/monaco-vscode-configuration-service-override'
import getExplorerServiceOverride from '@codingame/monaco-vscode-explorer-service-override'
import getExtensionServiceOverride from '@codingame/monaco-vscode-extensions-service-override'
import getFilesServiceOverride, {
  FileChangeType,
  FileSystemProviderError,
  FileSystemProviderErrorCode,
  FileType,
  RegisteredFileSystemProvider,
  RegisteredMemoryFile,
  registerFileSystemOverlay
} from '@codingame/monaco-vscode-files-service-override'
import getKeybindingsServiceOverride, {
  initUserKeybindings
} from '@codingame/monaco-vscode-keybindings-service-override'
import getLanguagesServiceOverride from '@codingame/monaco-vscode-languages-service-override'
import getModelServiceOverride from '@codingame/monaco-vscode-model-service-override'
import getStorageServiceOverride from '@codingame/monaco-vscode-storage-service-override'
import getTextmateServiceOverride from '@codingame/monaco-vscode-textmate-service-override'
import getThemeServiceOverride from '@codingame/monaco-vscode-theme-service-override'
import getWorkbenchServiceOverride from '@codingame/monaco-vscode-workbench-service-override'
import * as monaco from 'monaco-editor'
import { useEffect, useRef, useState } from 'react'

import floraSdkGlobalTypes from '../../../../../packages/sdk/global-types.d.ts?raw'
import { getParentFolder, normalizePath } from './editor-utils'

const WORKSPACE_ROOT = '/workspace'
const SUPPORT_FILES: Record<string, string> = {
  'node_modules/@uwu/flora-sdk/global-types.d.ts': floraSdkGlobalTypes,
  'node_modules/@uwu/flora-sdk/index.d.ts': `declare module '@uwu/flora-sdk' {
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
}
const SUPPORT_PATHS = new Set(Object.keys(SUPPORT_FILES))

const workerLoaders: Partial<Record<string, () => Worker>> = {
  editorWorkerService: () =>
    new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url), {
      type: 'module'
    }),
  TextMateWorker: () =>
    new Worker(
      new URL('@codingame/monaco-vscode-textmate-service-override/worker', import.meta.url),
      { type: 'module' }
    )
}

const fileWriteOptions: IFileWriteOptions = {
  create: true,
  overwrite: true,
  unlock: false,
  atomic: false
}

const textEncoder = new TextEncoder()

function ensureWorkerEnvironment() {
  if (globalThis.MonacoEnvironment) return
  Object.assign(globalThis, {
    MonacoEnvironment: {
      getWorker(_moduleId: string, label: string) {
        const loader = workerLoaders[label] ?? workerLoaders.editorWorkerService
        if (!loader) throw new Error(`Missing Monaco worker: ${label}`)
        return loader()
      }
    }
  })
}

function toWorkspaceUri(path: string) {
  return monaco.Uri.file(`${WORKSPACE_ROOT}/${normalizePath(path)}`)
}

function fromWorkspaceUri(uri: monaco.Uri) {
  if (uri.scheme !== 'file') return ''
  if (!uri.path.startsWith(WORKSPACE_ROOT)) return ''
  return normalizePath(uri.path.replace(`${WORKSPACE_ROOT}/`, ''))
}

async function listWorkspaceFiles(
  provider: RegisteredFileSystemProvider,
  root = WORKSPACE_ROOT
): Promise<string[]> {
  const entries = await provider.readdir(monaco.Uri.file(root))
  const files: string[] = []

  for (const [name, type] of entries) {
    const nextPath = `${root}/${name}`
    if (type === FileType.Directory) {
      const nested = await listWorkspaceFiles(provider, nextPath)
      files.push(...nested)
      continue
    }
    if (type === FileType.File) files.push(nextPath)
  }

  return files
}

async function ensureDirectory(provider: RegisteredFileSystemProvider, filePath: string) {
  const parent = getParentFolder(filePath)
  if (!parent) return
  const parts = normalizePath(parent).split('/').filter(Boolean)
  let current = WORKSPACE_ROOT
  for (const part of parts) {
    current = `${current}/${part}`
    try {
      provider.mkdirSync(monaco.Uri.file(current))
    } catch {
      // ignore existing dirs
    }
  }
}

async function upsertFile(provider: RegisteredFileSystemProvider, path: string, contents: string) {
  const uri = toWorkspaceUri(path)
  try {
    await provider.stat(uri)
    await provider.writeFile(uri, textEncoder.encode(contents), fileWriteOptions)
  } catch (error) {
    if (
      error instanceof FileSystemProviderError &&
      error.code === FileSystemProviderErrorCode.FileNotFound
    ) {
      provider.registerFile(new RegisteredMemoryFile(uri, contents))
      return
    }
    throw error
  }
}

function areFileMapsEqual(a: Record<string, string>, b: Record<string, string>) {
  const aKeys = Object.keys(a)
  const bKeys = Object.keys(b)
  if (aKeys.length !== bKeys.length) return false

  for (const key of aKeys) {
    if (!(key in b)) return false
    if (a[key] !== b[key]) return false
  }

  return true
}

type WorkbenchController = {
  provider: RegisteredFileSystemProvider
  syncFiles: (nextFiles: Record<string, string>) => Promise<void>
  subscribe: (listener: (next: Record<string, string>) => void) => () => void
}

let sharedController: WorkbenchController | null = null
let currentFiles: Record<string, string> = {}
let suppressEvents = false

async function createController(
  container: HTMLElement,
  initialFiles: Record<string, string>,
  entryFile?: string | null
) {
  ensureWorkerEnvironment()
  const provider = new RegisteredFileSystemProvider(false)
  registerFileSystemOverlay(1, provider)

  for (const [path, contents] of Object.entries(SUPPORT_FILES)) {
    await ensureDirectory(provider, path)
    await upsertFile(provider, path, contents)
  }

  for (const [path, contents] of Object.entries(initialFiles)) {
    const normalized = normalizePath(path)
    await ensureDirectory(provider, normalized)
    await upsertFile(provider, normalized, contents)
  }

  currentFiles = { ...initialFiles }

  const configDefaults = {
    'workbench.colorTheme': 'Default Dark+',
    'editor.fontSize': 14,
    'editor.minimap.enabled': false,
    'files.autoSave': 'off'
  }

  await Promise.all([
    initUserConfiguration(JSON.stringify(configDefaults, null, 2)),
    initUserKeybindings('[]')
  ])

  const workbenchOptions: IWorkbenchConstructionOptions = {
    workspaceProvider: {
      trusted: true,
      async open() {
        return true
      },
      workspace: {
        folderUri: monaco.Uri.file(WORKSPACE_ROOT)
      }
    },
    configurationDefaults: configDefaults,
    developmentOptions: {
      logLevel: LogLevel.Info
    },
    defaultLayout: entryFile
      ? {
          editors: [
            {
              uri: toWorkspaceUri(entryFile),
              viewColumn: 1
            }
          ],
          force: true
        }
      : undefined
  }

  const initializePromise = initialize(
    {
      ...getConfigurationServiceOverride(),
      ...getKeybindingsServiceOverride(),
      ...getModelServiceOverride(),
      ...getTextmateServiceOverride(),
      ...getThemeServiceOverride(),
      ...getLanguagesServiceOverride(),
      ...getFilesServiceOverride(),
      ...getStorageServiceOverride(),
      ...getExtensionServiceOverride({
        enableWorkerExtensionHost: false
      }),
      ...getExplorerServiceOverride(),
      ...getWorkbenchServiceOverride()
    },
    container,
    workbenchOptions
  )

  await initializePromise

  void registerExtension(
    {
      name: 'flora-workbench',
      publisher: 'flora',
      version: '1.0.0',
      engines: {
        vscode: '*'
      }
    },
    ExtensionHostKind.LocalProcess
  ).setAsDefaultApi()

  const controller: WorkbenchController = {
    provider,
    syncFiles: async (nextFiles) => {
      if (areFileMapsEqual(currentFiles, nextFiles)) return
      suppressEvents = true

      try {
        const existingPaths = await listWorkspaceFiles(provider)
        const nextPaths = new Set(
          Object.keys(nextFiles).map((path) => `${WORKSPACE_ROOT}/${normalizePath(path)}`)
        )

        for (const existingPath of existingPaths) {
          const relative = normalizePath(existingPath.replace(`${WORKSPACE_ROOT}/`, ''))
          if (SUPPORT_PATHS.has(relative)) continue
          if (nextPaths.has(existingPath)) continue
          await provider.delete(monaco.Uri.file(existingPath))
        }

        for (const [path, contents] of Object.entries(nextFiles)) {
          const normalized = normalizePath(path)
          await ensureDirectory(provider, normalized)
          await upsertFile(provider, normalized, contents)
        }

        currentFiles = { ...nextFiles }
      } finally {
        suppressEvents = false
      }
    },
    subscribe: (listener) => {
      listeners.add(listener)
      return () => listeners.delete(listener)
    }
  }

  return controller
}

const listeners = new Set<(next: Record<string, string>) => void>()

function emitFiles(next: Record<string, string>) {
  for (const listener of listeners) listener(next)
}

async function handleFileChange(
  provider: RegisteredFileSystemProvider,
  changes: readonly { type: FileChangeType; resource: monaco.Uri }[]
) {
  if (suppressEvents) return
  let changed = false

  for (const change of changes) {
    const path = fromWorkspaceUri(change.resource)
    if (!path || SUPPORT_PATHS.has(path)) continue

    if (change.type === FileChangeType.DELETED) {
      if (path in currentFiles) {
        const next = { ...currentFiles }
        delete next[path]
        currentFiles = next
        changed = true
      }
      continue
    }

    if (change.type === FileChangeType.ADDED || change.type === FileChangeType.UPDATED) {
      const content = await provider.readFile(change.resource)
      const text = new TextDecoder().decode(content)
      if (currentFiles[path] !== text) {
        currentFiles = { ...currentFiles, [path]: text }
        changed = true
      }
    }
  }

  if (changed) emitFiles({ ...currentFiles })
}

type EditorWorkbenchProps = {
  files: Record<string, string>
  entryFile?: string | null
  onFilesChange: (next: Record<string, string>) => void
}

export function EditorWorkbench({ files, entryFile, onFilesChange }: EditorWorkbenchProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const [ready, setReady] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const container = containerRef.current
    if (!container || Object.keys(files).length === 0) return

    let changeDisposable: IDisposable | null = null
    let unsubscribe: (() => void) | null = null

    const setup = async () => {
      try {
        if (!sharedController) {
          sharedController = await createController(container, files, entryFile)
        }

        changeDisposable = sharedController.provider.onDidChangeFile((changes) => {
          void handleFileChange(sharedController!.provider, changes)
        })

        void sharedController.syncFiles(files)
        unsubscribe = sharedController.subscribe(onFilesChange)
        setReady(true)
        setError(null)
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err)
        console.error('Workbench init failed', err)
        setError(message)
      }
    }

    void setup()

    return () => {
      changeDisposable?.dispose()
      if (unsubscribe) unsubscribe()
    }
  }, [entryFile, files, onFilesChange])

  return (
    <div className='relative h-full min-w-0 flex-1'>
      {!ready && !error && (
        <div className='absolute inset-0 z-10 flex items-center justify-center text-sm text-muted-foreground'>
          Loading workbench...
        </div>
      )}
      {error && (
        <div className='absolute inset-0 z-10 flex items-center justify-center px-6 text-center text-xs text-destructive'>
          {`Workbench failed: ${error}`}
        </div>
      )}
      <div ref={containerRef} className='h-full w-full' />
    </div>
  )
}
