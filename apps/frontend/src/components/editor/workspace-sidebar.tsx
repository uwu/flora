import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import {
  Briefcase,
  ChevronDown,
  ChevronRight,
  Copy,
  FileCode2,
  FileJson2,
  FileText,
  Folder,
  FolderOpen,
  Upload
} from 'lucide-react'
import type { MouseEvent as ReactMouseEvent, RefObject } from 'react'
import { match, P } from 'ts-pattern'

import { formatLogLine, getLanguageFromPath } from './editor-utils'
import type { FileTreeNode, TreeSelection } from './types'

type RuntimeLogLine = {
  timestamp: number
  target: string
  level: string
  message: string
}

type WorkspaceSidebarProps = {
  filesSectionOpen: boolean
  onFilesSectionOpenChange: (open: boolean) => void
  logsSectionOpen: boolean
  onLogsSectionOpenChange: (open: boolean) => void
  fileTree: FileTreeNode[]
  openFolders: Record<string, boolean>
  selectedTreeNode: TreeSelection | null
  selectedFile: string
  onFolderClick: (path: string, isOpen: boolean) => void
  onFileClick: (path: string) => void
  onOpenContextMenu: (event: ReactMouseEvent, target: TreeSelection | null) => void
  deployError: string | null
  deployButtonClass: string
  deployLabel: string
  hasUnsavedChanges: boolean
  deployUploadedFiles: string[]
  deployBuildLogs: string[]
  copyState: 'idle' | 'copied'
  guildId?: string
  fileCount: number
  isDeploying: boolean
  onDeploy: () => void
  onCopyDeployDetails: () => void
  tailLogs: boolean
  onTailLogsChange: (next: boolean) => void
  logsAreaRef: RefObject<HTMLDivElement | null>
  logsState: {
    isLoading: boolean
    isError: boolean
    data?: RuntimeLogLine[]
  }
}

export function WorkspaceSidebar({
  filesSectionOpen,
  onFilesSectionOpenChange,
  logsSectionOpen,
  onLogsSectionOpenChange,
  fileTree,
  openFolders,
  selectedTreeNode,
  selectedFile,
  onFolderClick,
  onFileClick,
  onOpenContextMenu,
  deployError,
  deployButtonClass,
  deployLabel,
  hasUnsavedChanges,
  deployUploadedFiles,
  deployBuildLogs,
  copyState,
  guildId,
  fileCount,
  isDeploying,
  onDeploy,
  onCopyDeployDetails,
  tailLogs,
  onTailLogsChange,
  logsAreaRef,
  logsState
}: WorkspaceSidebarProps) {
  const renderTreeNode = (node: FileTreeNode, depth: number) => {
    if (node.kind === 'folder') {
      const isOpen = openFolders[node.path] ?? node.path === 'src'
      const isSelected = selectedTreeNode?.kind === 'folder' && selectedTreeNode.path === node.path

      return (
        <div key={node.path}>
          <button
            type='button'
            onClick={() => onFolderClick(node.path, isOpen)}
            onContextMenu={(event) => onOpenContextMenu(event, { kind: 'folder', path: node.path })}
            className={cn(
              'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs transition-colors',
              isSelected
                ? 'bg-primary/10 text-primary'
                : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
            )}
            style={{ paddingLeft: `${8 + depth * 14}px` }}
          >
            {isOpen
              ? <FolderOpen className='h-3.5 w-3.5 shrink-0' />
              : <Folder className='h-3.5 w-3.5 shrink-0' />}
            <span className='truncate'>{node.name}</span>
          </button>
          {isOpen && node.children?.map((child) => renderTreeNode(child, depth + 1))}
        </div>
      )
    }

    const language = getLanguageFromPath(node.path)
    return (
      <button
        key={node.path}
        type='button'
        onClick={() => onFileClick(node.path)}
        onContextMenu={(event) => onOpenContextMenu(event, { kind: 'file', path: node.path })}
        className={cn(
          'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs transition-colors',
          selectedFile === node.path
            ? 'bg-primary/10 text-primary'
            : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
        )}
        style={{ paddingLeft: `${8 + depth * 14}px` }}
      >
        {language === 'json'
          ? <FileJson2 className='h-3.5 w-3.5 shrink-0' />
          : language === 'plaintext'
          ? <FileText className='h-3.5 w-3.5 shrink-0' />
          : <FileCode2 className='h-3.5 w-3.5 shrink-0' />}
        <span className='truncate'>{node.name}</span>
      </button>
    )
  }

  const uploadedFilesContent = match(deployUploadedFiles.length)
    .with(0, () => <div>(none)</div>)
    .otherwise(() => deployUploadedFiles.map((path) => <div key={path}>{path}</div>))

  const buildLogsContent = match(deployBuildLogs.length)
    .with(0, () => <div>(none)</div>)
    .otherwise(() => deployBuildLogs.map((line) => <div key={line}>{line}</div>))

  const deployAction = match(deployError)
    .with(P.string, (errorMessage) => (
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <button
              type='button'
              className={deployButtonClass}
            >
              <Upload className='h-3 w-3' />
              {deployLabel}
            </button>
          }
        />
        <DropdownMenuContent align='end' className='w-[32rem] p-3'>
          <div className='space-y-3'>
            <div>
              <div className='text-[11px] font-semibold uppercase tracking-wide text-muted-foreground'>
                Error
              </div>
              <div className='mt-1 max-h-28 overflow-auto rounded border bg-muted/40 p-2 font-mono text-xs text-destructive'>
                {errorMessage}
              </div>
            </div>
            <div>
              <div className='text-[11px] font-semibold uppercase tracking-wide text-muted-foreground'>
                Uploaded Files
              </div>
              <div className='mt-1 max-h-28 overflow-auto rounded border bg-muted/40 p-2 font-mono text-xs'>
                {uploadedFilesContent}
              </div>
            </div>
            <div>
              <div className='text-[11px] font-semibold uppercase tracking-wide text-muted-foreground'>
                Build Logs
              </div>
              <div className='mt-1 max-h-40 overflow-auto rounded border bg-muted/40 p-2 font-mono text-xs'>
                {buildLogsContent}
              </div>
            </div>
            <div className='flex justify-end gap-2'>
              <button
                type='button'
                onClick={onDeploy}
                disabled={!guildId || fileCount === 0 || isDeploying}
                className='inline-flex items-center gap-1 rounded border px-2 py-1 text-xs font-medium hover:bg-accent disabled:opacity-50'
              >
                <Upload className='h-3.5 w-3.5' />
                Retry
              </button>
              <button
                type='button'
                onClick={onCopyDeployDetails}
                className='inline-flex items-center gap-1 rounded border px-2 py-1 text-xs font-medium hover:bg-accent'
              >
                <Copy className='h-3.5 w-3.5' />
                {copyState === 'copied' ? 'Copied' : 'Copy'}
              </button>
            </div>
          </div>
        </DropdownMenuContent>
      </DropdownMenu>
    ))
    .otherwise(() => (
      <button
        type='button'
        onClick={onDeploy}
        disabled={!guildId || fileCount === 0 || isDeploying}
        className={deployButtonClass}
      >
        <Upload className='h-3 w-3' />
        {deployLabel}
      </button>
    ))

  return (
    <div className='h-full w-72 border-r bg-muted/10'>
      <div className='flex h-12 items-center justify-between px-2'>
        <div className='flex items-center gap-2 text-sm font-medium'>
          <Briefcase className='h-4 w-4 text-muted-foreground' />
          Workspace
        </div>
        <div className='flex items-center gap-1'>
          {hasUnsavedChanges && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger
                  render={
                    <button
                      type='button'
                      className='inline-flex items-center rounded-full border border-amber-500/40 bg-amber-500/10 px-2 py-0.5 text-[10px] font-semibold text-amber-600'
                    >
                      Unsaved
                    </button>
                  }
                />
                <TooltipContent side='bottom' align='end'>
                  You have local edits. Deploy to save and apply them.
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
          {deployAction}
        </div>
      </div>
      <Separator />
      <div className='h-[calc(100%-3.05rem)]'>
        <div className='flex h-full min-h-0 flex-col'>
          <Collapsible
            open={filesSectionOpen}
            onOpenChange={onFilesSectionOpenChange}
            className='flex min-h-0 flex-1 flex-col'
          >
            <CollapsibleTrigger
              render={
                <button
                  type='button'
                  className='flex h-8 w-full items-center gap-1 px-2 text-[11px] font-semibold tracking-wide text-muted-foreground hover:text-foreground'
                >
                  {filesSectionOpen
                    ? <ChevronDown className='h-3.5 w-3.5 shrink-0' />
                    : <ChevronRight className='h-3.5 w-3.5 shrink-0' />}
                  FILES
                </button>
              }
            />
            <CollapsibleContent className='min-h-0 flex-1'>
              <ScrollArea
                className='h-full px-2 py-1'
                onContextMenu={(event) => onOpenContextMenu(event, null)}
              >
                <div className='space-y-0.5'>
                  {fileTree.map((node) => renderTreeNode(node, 0))}
                </div>
              </ScrollArea>
            </CollapsibleContent>
          </Collapsible>

          <Separator className='my-1 shrink-0' />

          <Collapsible
            open={logsSectionOpen}
            onOpenChange={onLogsSectionOpenChange}
            className='mt-auto shrink-0'
          >
            <CollapsibleTrigger
              render={
                <div className='flex items-center justify-between gap-2 px-2'>
                  <button
                    type='button'
                    className='flex h-8 items-center gap-1 text-[11px] font-semibold tracking-wide text-muted-foreground hover:text-foreground'
                  >
                    {logsSectionOpen
                      ? <ChevronDown className='h-3.5 w-3.5 shrink-0' />
                      : <ChevronRight className='h-3.5 w-3.5 shrink-0' />}
                    LOGS
                  </button>
                  <button
                    type='button'
                    onClick={(event) => {
                      event.preventDefault()
                      event.stopPropagation()
                      onTailLogsChange(!tailLogs)
                    }}
                    className={cn(
                      'rounded px-1.5 py-0.5 text-[10px] font-medium transition-colors',
                      tailLogs
                        ? 'bg-primary/10 text-primary'
                        : 'bg-muted text-muted-foreground hover:text-foreground'
                    )}
                    title='Toggle log tailing'
                  >
                    {tailLogs ? 'Tail On' : 'Tail Off'}
                  </button>
                </div>
              }
            />
            <CollapsibleContent>
              <ScrollArea ref={logsAreaRef} className='h-48 px-3 py-2'>
                <div className='space-y-2 font-mono text-[11px]'>
                  {logsState.isLoading && (
                    <div className='rounded-md border bg-background/70 px-2 py-1 text-muted-foreground'>
                      Loading logs...
                    </div>
                  )}
                  {logsState.isError && (
                    <div className='rounded-md border bg-background/70 px-2 py-1 text-destructive'>
                      Failed to load logs.
                    </div>
                  )}
                  {logsState.data?.map((line) => (
                    <div
                      key={`${line.timestamp}-${line.target}-${line.level}-${line.message}`}
                      className='rounded-md border bg-background/70 px-2 py-1'
                    >
                      {formatLogLine(line)}
                    </div>
                  ))}
                </div>
              </ScrollArea>
            </CollapsibleContent>
          </Collapsible>
        </div>
      </div>
    </div>
  )
}
