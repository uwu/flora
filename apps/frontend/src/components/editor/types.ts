export type FileTreeNode = {
  kind: 'file' | 'folder'
  name: string
  path: string
  children?: FileTreeNode[]
}

export type TreeSelection = {
  kind: 'file' | 'folder'
  path: string
}

export type ContextMenuState = {
  x: number
  y: number
  target: TreeSelection | null
}

export type TextActionModalState = {
  mode: 'create_file' | 'create_folder' | 'rename'
  target: TreeSelection | null
  value: string
}

export type DeploymentFileRecord = Record<string, string>
