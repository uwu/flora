import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'
import { Copy, Upload } from 'lucide-react'
import type { RefObject } from 'react'
import { match, P } from 'ts-pattern'

import { formatLogLine } from './editor-utils'

type RuntimeLogLine = {
  timestamp: number
  target: string
  level: string
  message: string
}

type EditorSidePanelProps = {
  logsSectionOpen: boolean
  onLogsSectionOpenChange: (open: boolean) => void
  deployError: string | null
  deployButtonClass: string
  deployLabel: string
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

export function EditorSidePanel({
  logsSectionOpen,
  onLogsSectionOpenChange,
  deployError,
  deployButtonClass,
  deployLabel,
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
}: EditorSidePanelProps) {
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
    <div className='h-full w-72 border-l bg-muted/10'>
      <div className='flex h-12 items-center justify-between px-3'>
        <div className='text-sm font-medium'>Deploy</div>
        <div className='flex items-center gap-1'>
          {deployAction}
        </div>
      </div>
      <Separator />
      <div className='h-[calc(100%-3.05rem)]'>
        <div className='flex h-full min-h-0 flex-col'>
          <Collapsible
            open={logsSectionOpen}
            onOpenChange={onLogsSectionOpenChange}
            className='mt-auto shrink-0'
          >
            <CollapsibleTrigger
              render={
                <div className='flex items-center justify-between gap-2 px-3'>
                  <button
                    type='button'
                    className='flex h-8 items-center gap-1 text-[11px] font-semibold tracking-wide text-muted-foreground hover:text-foreground'
                  >
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
              <ScrollArea ref={logsAreaRef} className='h-64 px-3 py-2'>
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
