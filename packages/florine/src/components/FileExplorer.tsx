import { FileTree as PierreFileTree, useFileTree } from '@pierre/trees/react'
import { useEffect, useMemo, useRef } from 'react'

import type { FileTreeOptions } from '@pierre/trees'

export type FileExplorerProps = {
  paths: readonly string[]
  activePath: string | null
  onOpenFile: (path: string) => void
  onRenamePath: (sourcePath: string, destinationPath: string) => void
}

export function FileExplorer(props: FileExplorerProps) {
  const { activePath, onOpenFile, onRenamePath, paths } = props
  const pathsRef = useRef(new Set(paths))
  pathsRef.current = new Set(paths)

  const options = useMemo<FileTreeOptions>(
    () => ({
      paths,
      initialExpansion: 'open',
      flattenEmptyDirectories: true,
      search: true,
      density: 'compact',
      renaming: {
        onRename(event) {
          onRenamePath(event.sourcePath, event.destinationPath)
        }
      },
      onSelectionChange(selectedPaths) {
        const [path] = selectedPaths
        if (path && pathsRef.current.has(path)) onOpenFile(path)
      }
    }),
    [onOpenFile, onRenamePath, paths]
  )
  const { model } = useFileTree(options)

  useEffect(() => {
    model.resetPaths(paths)
  }, [model, paths])

  useEffect(() => {
    if (activePath) model.setSearch(null)
  }, [model, activePath])

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
