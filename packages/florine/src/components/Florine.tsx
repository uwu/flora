import { DockviewReact, type DockviewApi, type IDockviewPanelProps } from 'dockview'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { FileExplorer } from './FileExplorer.tsx'
import { FlorineEditor } from './FlorineEditor.tsx'
import { defaultFiles, normalizeFiles, renameFiles } from '../core/files.ts'
import {
  basename,
  comparePaths,
  isString,
  joinClasses,
  normalizePath,
  renameOpenPath
} from '../utils/path.ts'

import type { CSSProperties, FunctionComponent } from 'react'
import type { FlorineFile, FlorineFileMap } from '../core/files.ts'

export type FlorineProps = {
  files?: readonly FlorineFile[] | FlorineFileMap
  defaultActivePath?: string
  className?: string
  style?: CSSProperties
  onFilesChange?: (files: readonly FlorineFile[]) => void
}

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
    Record<string, FunctionComponent<IDockviewPanelProps<{ path: string }>>>
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
