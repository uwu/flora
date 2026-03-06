import { FilePenLine, FilePlus2, FolderPlus, Trash2 } from 'lucide-react'
import type { RefObject } from 'react'

import type { ContextMenuState, TreeSelection } from './types'

type TreeContextMenuProps = {
  contextMenu: ContextMenuState
  contextMenuRef: RefObject<HTMLDivElement | null>
  contextTarget: TreeSelection | null
  onClose: () => void
  onCreateFile: (target: TreeSelection | null) => void
  onCreateFolder: (target: TreeSelection | null) => void
  onRename: (target: TreeSelection | null) => void
  onDelete: (target: TreeSelection | null) => void
}

export function TreeContextMenu({
  contextMenu,
  contextMenuRef,
  contextTarget,
  onClose,
  onCreateFile,
  onCreateFolder,
  onRename,
  onDelete
}: TreeContextMenuProps) {
  return (
    <div
      ref={contextMenuRef}
      className='fixed z-50 min-w-48 rounded-xl border bg-popover p-1 text-popover-foreground shadow-2xl'
      style={{ left: contextMenu.x, top: contextMenu.y }}
    >
      <button
        type='button'
        className='flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs hover:bg-accent'
        onClick={() => {
          onCreateFile(contextTarget)
          onClose()
        }}
      >
        <FilePlus2 className='h-3.5 w-3.5' />
        New File
      </button>
      <button
        type='button'
        className='flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs hover:bg-accent'
        onClick={() => {
          onCreateFolder(contextTarget)
          onClose()
        }}
      >
        <FolderPlus className='h-3.5 w-3.5' />
        New Folder
      </button>
      <div className='my-1 h-px bg-border/70' />
      <button
        type='button'
        disabled={!contextTarget}
        className='flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs hover:bg-accent disabled:opacity-40'
        onClick={() => {
          onRename(contextTarget)
          onClose()
        }}
      >
        <FilePenLine className='h-3.5 w-3.5' />
        Rename
      </button>
      <button
        type='button'
        disabled={!contextTarget}
        className='flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs text-destructive hover:bg-destructive/10 disabled:opacity-40'
        onClick={() => {
          onDelete(contextTarget)
          onClose()
        }}
      >
        <Trash2 className='h-3.5 w-3.5' />
        Delete
      </button>
    </div>
  )
}
