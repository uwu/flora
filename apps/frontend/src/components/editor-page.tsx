import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { $api, api } from '@/lib/openapi-client'
import { useTheme } from '@/lib/theme'
import { cn } from '@/lib/utils'
import { lazy as lazyMonaco, Workspace } from 'modern-monaco'
import type { MouseEvent as ReactMouseEvent } from 'react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useLocation, useParams, useSearch } from 'wouter'
import floraSdkGlobalTypes from '../../../../sdk/global-types.d.ts?raw'
import { runEditorBuildFlow } from './editor/deploy-flow'
import { EditorMainPane } from './editor/editor-main-pane'
import { DeleteConfirmModal, TextActionModal } from './editor/editor-modals'
import {
  buildFileTree,
  collectParentFolders,
  extractFilesFromDeployment,
  floraSdkModuleTypes,
  getParentFolder,
  normalizePath
} from './editor/editor-utils'
import { TreeContextMenu } from './editor/tree-context-menu'
import type { ContextMenuState, TextActionModalState, TreeSelection } from './editor/types'
import { WorkspaceSidebar } from './editor/workspace-sidebar'

const WORKSPACE_SUPPORT_FILES: Record<string, string> = {
  'node_modules/@uwu/flora-sdk/global-types.d.ts': floraSdkGlobalTypes,
  'node_modules/@uwu/flora-sdk/index.d.ts': floraSdkModuleTypes
}

const WORKSPACE_SUPPORT_PATHS = new Set(Object.keys(WORKSPACE_SUPPORT_FILES))

function toWorkspacePath(input: string) {
  return normalizePath(input.replace(/^file:\/\//, ''))
}

function dirname(path: string) {
  const normalized = normalizePath(path)
  const idx = normalized.lastIndexOf('/')
  return idx === -1 ? '' : normalized.slice(0, idx)
}

async function listWorkspaceFiles(workspace: Workspace, root = '/'): Promise<string[]> {
  const entries = await workspace.fs.readDirectory(root)
  const files: string[] = []

  for (const [name, type] of entries) {
    const nextPath = `${root}${root.endsWith('/') ? '' : '/'}${name}`
    if (type === 2) {
      const nested = await listWorkspaceFiles(workspace, `${nextPath}/`)
      files.push(...nested)
      continue
    }
    if (type === 1) files.push(normalizePath(nextPath))
  }

  return files
}

export function EditorPage() {
  'use no memo'
  const { guildId } = useParams<{ guildId: string }>()
  const { setView, setSelectedGuild } = useApp()
  const { theme } = useTheme()
  const [filesSectionOpen, setFilesSectionOpen] = useState(true)
  const [logsSectionOpen, setLogsSectionOpen] = useState(true)
  const [selectedFile, setSelectedFile] = useState('src/main.ts')
  const [selectedTreeNode, setSelectedTreeNode] = useState<TreeSelection | null>(null)
  const [explicitFolders, setExplicitFolders] = useState<string[]>([])
  const [fileContents, setFileContents] = useState<Record<string, string>>({})
  const [openFolders, setOpenFolders] = useState<Record<string, boolean>>({ src: true })
  const [isDeploying, setIsDeploying] = useState(false)
  const [deployState, setDeployState] = useState<'idle' | 'deploying' | 'success' | 'error'>('idle')
  const [deployError, setDeployError] = useState<string | null>(null)
  const [deployBuildLogs, setDeployBuildLogs] = useState<string[]>([])
  const [deployUploadedFiles, setDeployUploadedFiles] = useState<string[]>([])
  const [copyState, setCopyState] = useState<'idle' | 'copied'>('idle')
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null)
  const [textActionModal, setTextActionModal] = useState<TextActionModalState | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<TreeSelection | null>(null)
  const [tailLogs, setTailLogs] = useState(true)
  const [workspaceReady, setWorkspaceReady] = useState(false)
  const workspaceRef = useRef<Workspace | null>(null)
  const selectedFileRef = useRef(selectedFile)
  const logsAreaRef = useRef<HTMLDivElement | null>(null)
  const contextMenuRef = useRef<HTMLDivElement | null>(null)
  const search = useSearch()
  const [, setLocation] = useLocation()

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId)
    setView('editor')
  }, [guildId, setSelectedGuild, setView])

  const deploymentQuery = $api.useQuery(
    'get',
    '/deployments/{guild_id}',
    {
      params: {
        path: { guild_id: guildId ?? '' },
        query: { include_bundle: true }
      }
    },
    {
      enabled: !!guildId
    }
  )

  const logsQuery = $api.useQuery(
    'get',
    '/logs/{guild_id}',
    {
      params: {
        path: { guild_id: guildId ?? '' },
        query: { limit: 100 }
      }
    },
    {
      enabled: !!guildId,
      refetchInterval: 3000
    }
  )

  const filesFromDeployment = useMemo(
    () => extractFilesFromDeployment(deploymentQuery.data),
    [deploymentQuery.data]
  )

  useEffect(() => {
    if (Object.keys(filesFromDeployment).length === 0) return

    const timer = window.setTimeout(() => {
      setFileContents(filesFromDeployment)
      const folders = Array.from(
        new Set(
          Object.keys(filesFromDeployment)
            .flatMap((path) => collectParentFolders(path))
            .filter(Boolean)
        )
      )
      setExplicitFolders(folders)
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [filesFromDeployment])

  const files = useMemo(() => Object.keys(fileContents), [fileContents])
  const fileTree = useMemo(() => buildFileTree(files, explicitFolders), [explicitFolders, files])
  const preferredEntryFile = deploymentQuery.data?.entry
  const requestedFileFromUrl = useMemo(() => {
    const file = new URLSearchParams(search).get('file')
    return file ? normalizePath(file) : ''
  }, [search])

  useEffect(() => {
    if (!requestedFileFromUrl) return
    const timer = window.setTimeout(() => {
      setSelectedFile((prev) => (prev === requestedFileFromUrl ? prev : requestedFileFromUrl))
    }, 0)
    return () => {
      window.clearTimeout(timer)
    }
  }, [requestedFileFromUrl])

  useEffect(() => {
    if (!guildId || !selectedFile) return
    const params = new URLSearchParams(search)
    if (params.get('file') === selectedFile) return

    params.set('file', selectedFile)
    const query = params.toString()
    setLocation(`/${guildId}/editor${query ? `?${query}` : ''}`)
  }, [guildId, search, selectedFile, setLocation])

  useEffect(() => {
    if (files.length === 0) return
    if (files.includes(selectedFile)) return

    const preferredMatches = [
      preferredEntryFile,
      preferredEntryFile ? `src/${preferredEntryFile}` : null
    ].filter((value): value is string => !!value)

    const preferred = preferredMatches.find((entry) => files.includes(entry))
    const timer = window.setTimeout(() => {
      setSelectedFile(preferred ?? files[0])
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [files, preferredEntryFile, selectedFile])

  useEffect(() => {
    selectedFileRef.current = selectedFile
    if (!selectedFile) return

    const timer = window.setTimeout(() => {
      setSelectedTreeNode({ kind: 'file', path: selectedFile })
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [selectedFile])

  useEffect(() => {
    const parts = selectedFile.split('/').filter(Boolean)
    if (parts.length <= 1) return

    const next = { ...openFolders }
    let changed = false
    let currentPath = ''
    for (let i = 0; i < parts.length - 1; i += 1) {
      currentPath = currentPath ? `${currentPath}/${parts[i]}` : parts[i]
      if (!next[currentPath]) {
        next[currentPath] = true
        changed = true
      }
    }
    if (!changed) return

    const timer = window.setTimeout(() => {
      setOpenFolders(next)
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [openFolders, selectedFile])

  useEffect(() => {
    if (workspaceRef.current) return

    const workspace = new Workspace({
      name: guildId ? `flora-editor-${guildId}` : 'flora-editor',
      initialFiles: {
        ...WORKSPACE_SUPPORT_FILES,
        'src/main.ts': '// Loading workspace...'
      },
      entryFile: 'src/main.ts'
    })

    workspaceRef.current = workspace

    lazyMonaco({
      workspace,
      defaultTheme: 'vitesse-dark',
      lsp: {
        json: {
          allowComments: true,
          trailingCommas: 'ignore'
        },
        typescript: {
          compilerOptions: {
            allowNonTsExtensions: true,
            allowSyntheticDefaultImports: true,
            esModuleInterop: true,
            resolveJsonModule: true,
            allowJs: true,
            checkJs: true
          }
        }
      }
    })

    const timer = window.setTimeout(() => {
      setWorkspaceReady(true)
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [guildId])

  useEffect(() => {
    if (!workspaceReady) return
    const workspace = workspaceRef.current
    if (!workspace) return
    if (Object.keys(filesFromDeployment).length === 0) return

    let cancelled = false

    const syncWorkspace = async () => {
      const existingPaths = await listWorkspaceFiles(workspace)
      const nextPaths = new Set(Object.keys(filesFromDeployment))

      for (const existingPath of existingPaths) {
        if (WORKSPACE_SUPPORT_PATHS.has(existingPath)) continue
        if (nextPaths.has(existingPath)) continue
        await workspace.fs.delete(existingPath)
      }

      for (const [path, contents] of Object.entries(filesFromDeployment)) {
        const folder = dirname(path)
        if (folder) await workspace.fs.createDirectory(folder)
        await workspace.fs.writeFile(path, contents)
      }

      if (cancelled) return
      await workspace.openTextDocument(selectedFileRef.current)
    }

    void syncWorkspace().catch(() => {})

    return () => {
      cancelled = true
    }
  }, [filesFromDeployment, workspaceReady])

  useEffect(() => {
    if (!workspaceReady) return
    const workspace = workspaceRef.current
    if (!workspace) return

    const unwatch = workspace.fs.watch('/', { recursive: true }, (kind, filename, type) => {
      const path = toWorkspacePath(filename)
      if (!path || WORKSPACE_SUPPORT_PATHS.has(path)) return

      if (kind === 'remove') {
        setFileContents((prev) => {
          if (!(path in prev) && !Object.keys(prev).some((key) => key.startsWith(`${path}/`))) {
            return prev
          }
          const next: Record<string, string> = {}
          for (const [entryPath, contents] of Object.entries(prev)) {
            if (entryPath === path || entryPath.startsWith(`${path}/`)) continue
            next[entryPath] = contents
          }
          return next
        })
        return
      }

      if (type !== 1) return

      void workspace.fs.readTextFile(path).then((contents) => {
        setFileContents((prev) => {
          if (prev[path] === contents) return prev
          return { ...prev, [path]: contents }
        })
      })
    })

    return () => {
      unwatch()
    }
  }, [workspaceReady])

  useEffect(() => {
    if (!workspaceReady || !selectedFile) return
    const workspace = workspaceRef.current
    if (!workspace) return
    void workspace.openTextDocument(selectedFile).catch(() => {})
  }, [selectedFile, workspaceReady])

  useEffect(() => {
    if (!workspaceReady) return
    const workspace = workspaceRef.current
    if (!workspace) return

    return workspace.history.onChange(({ current }) => {
      const path = toWorkspacePath(current)
      if (!path || WORKSPACE_SUPPORT_PATHS.has(path)) return
      setSelectedFile((prev) => (prev === path ? prev : path))
      setSelectedTreeNode({ kind: 'file', path })
    })
  }, [workspaceReady])

  useEffect(() => {
    const root = logsAreaRef.current
    if (!root) return
    const viewport = root.querySelector('[data-radix-scroll-area-viewport]') as
      | HTMLDivElement
      | null
    if (!viewport) return

    const onScroll = () => {
      const distanceToBottom = viewport.scrollHeight - (viewport.scrollTop + viewport.clientHeight)
      if (distanceToBottom > 16 && tailLogs) {
        setTailLogs(false)
      }
    }

    viewport.addEventListener('scroll', onScroll, { passive: true })
    return () => viewport.removeEventListener('scroll', onScroll)
  }, [tailLogs, logsSectionOpen])

  useEffect(() => {
    if (!tailLogs || !logsSectionOpen) return
    const root = logsAreaRef.current
    if (!root) return
    const viewport = root.querySelector('[data-radix-scroll-area-viewport]') as
      | HTMLDivElement
      | null
    if (!viewport) return
    viewport.scrollTop = viewport.scrollHeight
  }, [logsQuery.data, tailLogs, logsSectionOpen])

  useEffect(() => {
    if (!contextMenu) return

    const onPointerDown = (event: MouseEvent) => {
      if (!contextMenuRef.current) return
      if (contextMenuRef.current.contains(event.target as Node)) return
      setContextMenu(null)
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setContextMenu(null)
    }

    window.addEventListener('mousedown', onPointerDown)
    window.addEventListener('keydown', onKeyDown)
    return () => {
      window.removeEventListener('mousedown', onPointerDown)
      window.removeEventListener('keydown', onKeyDown)
    }
  }, [contextMenu])

  useEffect(() => {
    if (deployState !== 'success') return
    const timer = window.setTimeout(() => setDeployState('idle'), 1800)
    return () => window.clearTimeout(timer)
  }, [deployState])

  const handleFolderClick = (path: string, isOpen: boolean) => {
    setSelectedTreeNode({ kind: 'folder', path })
    setOpenFolders((prev) => ({
      ...prev,
      [path]: !isOpen
    }))
  }

  const handleFileClick = (path: string) => {
    setSelectedTreeNode({ kind: 'file', path })
    setSelectedFile(path)
  }

  const addOrOpenFolders = (paths: string[]) => {
    if (paths.length === 0) return
    const normalized = paths.map((path) => normalizePath(path)).filter(Boolean)
    setExplicitFolders((prev) => Array.from(new Set([...prev, ...normalized])))
    setOpenFolders((prev) => {
      const next = { ...prev }
      for (const path of normalized) {
        next[path] = true
      }
      return next
    })
  }

  const openContextMenu = (event: ReactMouseEvent, target: TreeSelection | null) => {
    event.preventDefault()
    event.stopPropagation()
    if (target) setSelectedTreeNode(target)
    setContextMenu({
      x: event.clientX,
      y: event.clientY,
      target
    })
  }

  const handleCreateFile = (target: TreeSelection | null = selectedTreeNode) => {
    const baseFolder = target?.kind === 'folder'
      ? target.path
      : getParentFolder(selectedFile) || 'src'
    setTextActionModal({
      mode: 'create_file',
      target,
      value: baseFolder ? `${baseFolder}/new-file.ts` : 'src/new-file.ts'
    })
    setContextMenu(null)
  }

  const handleCreateFolder = (target: TreeSelection | null = selectedTreeNode) => {
    const baseFolder = target?.kind === 'folder'
      ? target.path
      : getParentFolder(selectedFile) || 'src'
    setTextActionModal({
      mode: 'create_folder',
      target,
      value: baseFolder ? `${baseFolder}/new-folder` : 'src/new-folder'
    })
    setContextMenu(null)
  }

  const handleRenameSelected = (target: TreeSelection | null = selectedTreeNode) => {
    if (!target) return
    setTextActionModal({
      mode: 'rename',
      target,
      value: target.path
    })
    setContextMenu(null)
  }

  const handleDeleteSelected = (target: TreeSelection | null = selectedTreeNode) => {
    if (!target) return
    setDeleteTarget(target)
    setContextMenu(null)
  }

  const applyDelete = (target: TreeSelection) => {
    const workspace = workspaceRef.current
    if (workspace) {
      void workspace.fs.delete(
        target.path,
        target.kind === 'folder' ? { recursive: true } : undefined
      ).catch(() => {})
    }

    if (target.kind === 'file') {
      setFileContents((prev) => {
        if (!(target.path in prev)) return prev
        const rest = { ...prev }
        delete rest[target.path]
        const nextKeys = Object.keys(rest)
        if (selectedFile === target.path && nextKeys.length > 0) {
          setSelectedFile(nextKeys[0])
        }
        return rest
      })
      setSelectedTreeNode(null)
      return
    }

    setFileContents((prev) => {
      const next: Record<string, string> = {}
      for (const [path, contents] of Object.entries(prev)) {
        if (path === target.path || path.startsWith(`${target.path}/`)) continue
        next[path] = contents
      }
      const nextKeys = Object.keys(next)
      if (
        (selectedFile === target.path ||
          selectedFile.startsWith(`${target.path}/`)) &&
        nextKeys.length > 0
      ) {
        setSelectedFile(nextKeys[0])
      }
      return next
    })
    setExplicitFolders((prev) =>
      prev.filter((path) => path !== target.path && !path.startsWith(`${target.path}/`))
    )
    setOpenFolders((prev) => {
      const next: Record<string, boolean> = {}
      for (const [path, isOpen] of Object.entries(prev)) {
        if (path === target.path || path.startsWith(`${target.path}/`)) continue
        next[path] = isOpen
      }
      return next
    })
    setSelectedTreeNode(null)
  }

  const applyCreateFile = (pathInput: string) => {
    const normalized = normalizePath(pathInput)
    if (!normalized) return

    const workspace = workspaceRef.current
    if (workspace) {
      void (async () => {
        const folder = dirname(normalized)
        if (folder) await workspace.fs.createDirectory(folder)
        await workspace.fs.writeFile(normalized, '')
        await workspace.openTextDocument(normalized)
      })().catch(() => {})
    }

    setFileContents((prev) => {
      if (normalized in prev) return prev
      return { ...prev, [normalized]: '' }
    })
    addOrOpenFolders(collectParentFolders(normalized))
    setSelectedTreeNode({ kind: 'file', path: normalized })
    setSelectedFile(normalized)
  }

  const applyCreateFolder = (pathInput: string) => {
    const normalized = normalizePath(pathInput)
    if (!normalized) return

    const workspace = workspaceRef.current
    if (workspace) {
      void workspace.fs.createDirectory(normalized).catch(() => {})
    }

    addOrOpenFolders([normalized, ...collectParentFolders(normalized)])
    setSelectedTreeNode({ kind: 'folder', path: normalized })
  }

  const applyRename = (target: TreeSelection, pathInput: string) => {
    const nextPath = normalizePath(pathInput)
    if (!nextPath || nextPath === target.path) return

    const workspace = workspaceRef.current
    if (workspace) {
      void workspace.fs.rename(target.path, nextPath, { overwrite: false }).catch(() => {})
    }

    if (target.kind === 'file') {
      setFileContents((prev) => {
        if (!(target.path in prev)) return prev
        const next = { ...prev }
        const currentContents = next[target.path]
        delete next[target.path]
        next[nextPath] = currentContents
        return next
      })
      addOrOpenFolders(collectParentFolders(nextPath))
      if (selectedFile === target.path) setSelectedFile(nextPath)
      setSelectedTreeNode({ kind: 'file', path: nextPath })
      return
    }

    setFileContents((prev) => {
      const next: Record<string, string> = {}
      for (const [path, contents] of Object.entries(prev)) {
        if (path === target.path || path.startsWith(`${target.path}/`)) {
          const suffix = path.slice(target.path.length)
          next[`${nextPath}${suffix}`] = contents
        } else {
          next[path] = contents
        }
      }
      return next
    })

    setExplicitFolders((prev) =>
      Array.from(
        new Set(
          prev
            .map((path) => {
              if (path === target.path || path.startsWith(`${target.path}/`)) {
                const suffix = path.slice(target.path.length)
                return `${nextPath}${suffix}`
              }
              return path
            })
            .concat(collectParentFolders(nextPath))
            .map((path) => normalizePath(path))
            .filter(Boolean)
        )
      )
    )

    setOpenFolders((prev) => {
      const next: Record<string, boolean> = {}
      for (const [path, isOpen] of Object.entries(prev)) {
        if (path === target.path || path.startsWith(`${target.path}/`)) {
          const suffix = path.slice(target.path.length)
          next[`${nextPath}${suffix}`] = isOpen
        } else {
          next[path] = isOpen
        }
      }
      next[nextPath] = true
      return next
    })

    if (selectedFile === target.path || selectedFile.startsWith(`${target.path}/`)) {
      const suffix = selectedFile.slice(target.path.length)
      setSelectedFile(`${nextPath}${suffix}`)
    }
    setSelectedTreeNode({ kind: 'folder', path: nextPath })
  }

  const handleDeploy = () => {
    if (!guildId || isDeploying) return
    setDeployError(null)
    setDeployState('deploying')
    setCopyState('idle')
    setIsDeploying(true)
    setDeployBuildLogs([])

    const preferredEntry = deploymentQuery.data?.entry ?? 'src/main.ts'

    void runEditorBuildFlow({
      guildId,
      fileContents,
      preferredEntry,
      fallbackEntry: selectedFile,
      onBuildLog: (line) => setDeployBuildLogs((prev) => [...prev, line])
    })
      .then(async (buildResult) => {
        setDeployUploadedFiles(buildResult.uploadedFiles)

        const deployFiles = Object.entries(fileContents)
          .map(([path, contents]) => ({ path, contents }))
          .sort((a, b) => a.path.localeCompare(b.path))

        const deployResponse = await api.POST('/deployments/{guild_id}', {
          params: { path: { guild_id: buildResult.build.guild_id } },
          body: {
            entry: buildResult.build.entry,
            files: deployFiles,
            bundle: buildResult.build.artifact.bundle
          }
        })

        const deploymentError: unknown = deployResponse.error
        if (deploymentError) {
          const errorMessage = typeof deploymentError === 'string'
            ? deploymentError
            : JSON.stringify(deploymentError)
          setDeployError(errorMessage)
          setDeployState('error')
          return
        }

        setDeployBuildLogs([])
        setDeployState('success')
        await Promise.all([deploymentQuery.refetch(), logsQuery.refetch()])
      })
      .catch((error: unknown) => {
        const message = error instanceof Error ? error.message : 'Deploy failed'
        setDeployError(message)
        setDeployState('error')
      })
      .finally(() => {
        setIsDeploying(false)
      })
  }

  const handleCopyDeployDetails = async () => {
    const lines = [
      `Error: ${deployError ?? 'unknown'}`,
      '',
      'Uploaded files:',
      ...(deployUploadedFiles.length > 0 ? deployUploadedFiles : ['(none)']),
      '',
      'Build logs:',
      ...(deployBuildLogs.length > 0 ? deployBuildLogs : ['(none)'])
    ]
    const text = lines.join('\n')
    await navigator.clipboard.writeText(text)
    setCopyState('copied')
    window.setTimeout(() => setCopyState('idle'), 1200)
  }

  const submitTextAction = () => {
    if (!textActionModal) return
    const value = textActionModal.value.trim()
    if (!value) return

    if (textActionModal.mode === 'create_file') {
      applyCreateFile(value)
    } else if (textActionModal.mode === 'create_folder') {
      applyCreateFolder(value)
    } else if (textActionModal.mode === 'rename' && textActionModal.target) {
      applyRename(textActionModal.target, value)
    }

    setTextActionModal(null)
  }

  const contextTarget = contextMenu?.target ?? selectedTreeNode
  const deployLabel = deployState === 'deploying'
    ? 'Deploying...'
    : deployState === 'success'
    ? 'Deployed'
    : deployState === 'error'
    ? 'Deploy Failed'
    : 'Deploy'
  const deployButtonClass = cn(
    'inline-flex items-center gap-1 rounded-md border px-2 py-1 text-[10px] font-medium transition-all duration-300 disabled:opacity-50',
    deployState === 'deploying' &&
      'border-primary/40 bg-primary/10 text-primary shadow-[0_0_24px_-12px_hsl(var(--primary))] animate-pulse',
    deployState === 'success' &&
      'border-emerald-500/40 bg-emerald-500/10 text-emerald-600 shadow-[0_0_24px_-12px_rgba(16,185,129,0.85)]',
    deployState === 'error' &&
      'border-destructive/50 bg-destructive/10 text-destructive shadow-[0_0_24px_-12px_hsl(var(--destructive))]',
    deployState === 'idle' && 'hover:bg-accent'
  )

  return (
    <SidebarProvider>
      <div className='relative flex h-dvh w-full'>
        <DashboardSidebar />
        <SidebarInset className='flex min-w-0 flex-1'>
          <div className='flex h-full min-h-0 w-full overflow-hidden'>
            <WorkspaceSidebar
              filesSectionOpen={filesSectionOpen}
              onFilesSectionOpenChange={setFilesSectionOpen}
              logsSectionOpen={logsSectionOpen}
              onLogsSectionOpenChange={setLogsSectionOpen}
              fileTree={fileTree}
              openFolders={openFolders}
              selectedTreeNode={selectedTreeNode}
              selectedFile={selectedFile}
              onFolderClick={handleFolderClick}
              onFileClick={handleFileClick}
              onOpenContextMenu={openContextMenu}
              deployError={deployError}
              deployButtonClass={deployButtonClass}
              deployLabel={deployLabel}
              deployUploadedFiles={deployUploadedFiles}
              deployBuildLogs={deployBuildLogs}
              copyState={copyState}
              guildId={guildId}
              fileCount={files.length}
              isDeploying={isDeploying}
              onDeploy={handleDeploy}
              onCopyDeployDetails={handleCopyDeployDetails}
              tailLogs={tailLogs}
              onTailLogsChange={setTailLogs}
              logsAreaRef={logsAreaRef}
              logsState={{
                isLoading: logsQuery.isLoading,
                isError: logsQuery.isError,
                data: logsQuery.data
              }}
            />
            <EditorMainPane
              theme={theme}
              deploymentState={{
                isLoading: deploymentQuery.isLoading,
                isError: deploymentQuery.isError
              }}
            />
          </div>
          {contextMenu && (
            <TreeContextMenu
              contextMenu={contextMenu}
              contextMenuRef={contextMenuRef}
              contextTarget={contextTarget}
              onClose={() => setContextMenu(null)}
              onCreateFile={handleCreateFile}
              onCreateFolder={handleCreateFolder}
              onRename={handleRenameSelected}
              onDelete={handleDeleteSelected}
            />
          )}
          {textActionModal && (
            <TextActionModal
              state={textActionModal}
              onChangeValue={(value) => {
                setTextActionModal((prev) => (prev ? { ...prev, value } : prev))
              }}
              onClose={() => setTextActionModal(null)}
              onSubmit={submitTextAction}
            />
          )}
          {deleteTarget && (
            <DeleteConfirmModal
              target={deleteTarget}
              onClose={() => setDeleteTarget(null)}
              onConfirm={() => {
                applyDelete(deleteTarget)
                setDeleteTarget(null)
              }}
            />
          )}
        </SidebarInset>
      </div>
    </SidebarProvider>
  )
}
