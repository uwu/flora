import { Editor as MonacoEditor } from '@monaco-editor/react'
import { FileTree as PierreFileTree, useFileTree } from '@pierre/trees/react'
import { DockviewReact, type DockviewApi, type IDockviewPanelProps } from 'dockview'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import type { CSSProperties } from 'react'
import type { FileTreeOptions } from '@pierre/trees'
import type { OnChange } from '@monaco-editor/react'

export type FlorineFile = {
  path: string
  content: string
}

export type FlorineFileMap = Record<string, string>

export type FlorineProps = {
  files?: readonly FlorineFile[] | FlorineFileMap
  defaultActivePath?: string
  className?: string
  style?: CSSProperties
  onFilesChange?: (files: readonly FlorineFile[]) => void
}

export type FileExplorerProps = {
  paths: readonly string[]
  activePath: string | null
  onOpenFile: (path: string) => void
  onRenamePath: (sourcePath: string, destinationPath: string) => void
}

const defaultFiles: readonly FlorineFile[] = [
  {
    path: 'README.md',
    content: '# Florine\n\nA lightweight VS Code-inspired editor shell.\n'
  },
  {
    path: 'src/index.ts',
    content: "export const greeting = 'hello from Florine'\n"
  }
]

export function Florine(props: FlorineProps) {
  const [files, setFiles] = useState(() => normalizeFiles(props.files ?? defaultFiles))
  const [activePath, setActivePath] = useState(() => {
    const normalizedDefault = normalizePath(props.defaultActivePath ?? '')
    return normalizedDefault || files[0]?.path || null
  })
  const [openPaths, setOpenPaths] = useState<readonly string[]>(() =>
    activePath ? [activePath] : []
  )
  const dock = useRef<DockviewApi | null>(null)

  const filePaths = useMemo(() => files.map((file) => file.path).sort(comparePaths), [files])
  const filesByPath = useMemo(
    () => new Map(files.map((file) => [file.path, file.content])),
    [files]
  )

  const updateFiles = useCallback(
    (next: readonly FlorineFile[]) => {
      const normalized = normalizeFiles(next)
      setFiles(normalized)
      props.onFilesChange?.(normalized)
    },
    [props]
  )

  const openFile = useCallback(
    (path: string) => {
      const normalized = normalizePath(path)
      if (!normalized || !filesByPath.has(normalized)) return

      setActivePath(normalized)
      setOpenPaths((current) => (current.includes(normalized) ? current : [...current, normalized]))
    },
    [filesByPath]
  )

  const updateFileContent = useCallback(
    (path: string, content: string) => {
      updateFiles(files.map((file) => (file.path === path ? { ...file, content } : file)))
    },
    [files, updateFiles]
  )

  const renamePath = useCallback(
    (sourcePath: string, destinationPath: string) => {
      const source = normalizePath(sourcePath)
      const destination = normalizePath(destinationPath)
      if (!source || !destination || source === destination) return

      updateFiles(renameFiles(files, source, destination))
      setActivePath((current) => renameOpenPath(current, source, destination))
      setOpenPaths((current) =>
        current.map((path) => renameOpenPath(path, source, destination)).filter(isString)
      )
    },
    [files, updateFiles]
  )

  const components = useMemo<
    Record<string, React.FunctionComponent<IDockviewPanelProps<{ path: string }>>>
  >(
    () => ({
      editor: (panelProps) => (
        <FlorineEditor
          path={panelProps.params.path}
          content={filesByPath.get(panelProps.params.path) ?? ''}
          onChange={updateFileContent}
        />
      )
    }),
    [filesByPath, updateFileContent]
  )

  useEffect(() => {
    if (!activePath || filesByPath.has(activePath)) return
    const fallback = files[0]?.path ?? null
    setActivePath(fallback)
    setOpenPaths(fallback ? [fallback] : [])
  }, [activePath, files, filesByPath])

  useEffect(() => {
    if (!dock.current || !activePath) return

    const existing = dock.current.getPanel(activePath)
    if (existing) {
      existing.api.setActive()
      return
    }

    dock.current.addPanel({
      id: activePath,
      title: basename(activePath),
      component: 'editor',
      params: { path: activePath }
    })
  }, [activePath, openPaths])

  return (
    <section className={joinClasses('florine', props.className)} style={props.style}>
      <aside className='florine__explorer' aria-label='Explorer'>
        <FileExplorer
          paths={filePaths}
          activePath={activePath}
          onOpenFile={openFile}
          onRenamePath={renamePath}
        />
      </aside>
      <main className='florine__workbench'>
        <DockviewReact
          className='dockview-theme-dark florine__dock'
          components={components}
          onReady={(event) => {
            dock.current = event.api
            if (activePath) {
              event.api.addPanel({
                id: activePath,
                title: basename(activePath),
                component: 'editor',
                params: { path: activePath }
              })
            }
          }}
        />
      </main>
    </section>
  )
}

export function FileExplorer(props: FileExplorerProps) {
  const pathsRef = useRef(new Set(props.paths))
  pathsRef.current = new Set(props.paths)

  const options = useMemo<FileTreeOptions>(
    () => ({
      paths: props.paths,
      initialExpansion: 'open',
      flattenEmptyDirectories: true,
      search: true,
      density: 'compact',
      renaming: {
        onRename(event) {
          props.onRenamePath(event.sourcePath, event.destinationPath)
        }
      },
      onSelectionChange(selectedPaths) {
        const [path] = selectedPaths
        if (path && pathsRef.current.has(path)) props.onOpenFile(path)
      }
    }),
    [props]
  )
  const { model } = useFileTree(options)

  useEffect(() => {
    model.resetPaths(props.paths)
  }, [model, props.paths])

  useEffect(() => {
    if (props.activePath) model.setSearch(null)
  }, [model, props.activePath])

  return (
    <div className='florine__tree'>
      <PierreFileTree
        model={model}
        header={<div className='florine__tree-header'>Explorer</div>}
        style={{ height: '100%' }}
      />
    </div>
  )
}

export type FlorineEditorProps = {
  path: string
  content: string
  onChange: (path: string, content: string) => void
}

export function FlorineEditor(props: FlorineEditorProps) {
  const handleChange: OnChange = (value) => props.onChange(props.path, value ?? '')

  return (
    <MonacoEditor
      path={props.path}
      value={props.content}
      theme='vs-dark'
      language={languageForPath(props.path)}
      options={{
        automaticLayout: true,
        minimap: { enabled: false },
        padding: { top: 12 },
        scrollBeyondLastLine: false
      }}
      onChange={handleChange}
    />
  )
}

function normalizeFiles(files: readonly FlorineFile[] | FlorineFileMap): readonly FlorineFile[] {
  const entries = Array.isArray(files)
    ? files
    : Object.entries(files).map(([path, content]) => ({ path, content }))

  const deduped = new Map<string, string>()
  for (const file of entries) {
    const path = normalizePath(file.path)
    if (path) deduped.set(path, file.content)
  }

  return [...deduped.entries()]
    .map(([path, content]) => ({ path, content }))
    .sort((left, right) => comparePaths(left.path, right.path))
}

function renameFiles(
  files: readonly FlorineFile[],
  sourcePath: string,
  destinationPath: string
): readonly FlorineFile[] {
  const sourcePrefix = `${sourcePath}/`
  const next = files.map((file) => {
    if (file.path === sourcePath) return { ...file, path: destinationPath }
    if (file.path.startsWith(sourcePrefix)) {
      return { ...file, path: `${destinationPath}/${file.path.slice(sourcePrefix.length)}` }
    }
    return file
  })

  return normalizeFiles(next)
}

function renameOpenPath(
  path: string | null,
  sourcePath: string,
  destinationPath: string
): string | null {
  if (!path) return null
  if (path === sourcePath) return destinationPath

  const sourcePrefix = `${sourcePath}/`
  if (path.startsWith(sourcePrefix)) return `${destinationPath}/${path.slice(sourcePrefix.length)}`

  return path
}

function normalizePath(path: string) {
  return path.trim().replace(/^\.\//, '').replace(/^\//, '').replace(/\/$/, '')
}

function basename(path: string) {
  return path.split('/').at(-1) || path
}

function comparePaths(left: string, right: string) {
  return left.localeCompare(right, undefined, { sensitivity: 'base' })
}

function languageForPath(path: string) {
  const extension = path.split('.').at(-1)

  switch (extension) {
    case 'css':
      return 'css'
    case 'html':
      return 'html'
    case 'json':
      return 'json'
    case 'md':
    case 'mdx':
      return 'markdown'
    case 'ts':
    case 'tsx':
      return 'typescript'
    case 'js':
    case 'jsx':
      return 'javascript'
    default:
      return 'plaintext'
  }
}

function joinClasses(...classes: readonly (string | undefined)[]) {
  return classes.filter(isString).join(' ')
}

function isString(value: string | null | undefined): value is string {
  return typeof value === 'string' && value.length > 0
}
