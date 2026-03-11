import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from '@/components/ui/table'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { DeploymentRevision } from '@/data/deployments/types'
import { useRollbackDeploymentMutation } from '@/data/mutations'
import { useDeploymentHistoryQuery, useDeploymentRevisionQuery } from '@/data/queries'
import { cn } from '@/lib/utils'
import { formatDistanceToNow } from 'date-fns'
import { ChevronDown, Loader2, RotateCcw } from 'lucide-react'
import { lazy, Suspense, useEffect, useMemo, useState } from 'react'

const LazyMultiFileDiff = lazy(async () => {
  const mod = await import('@pierre/diffs/react')
  return { default: mod.MultiFileDiff }
})

const DIFFABLE_EXTENSIONS = new Set(['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cts'])
const DIFFABLE_FILENAMES = new Set([
  'package.json',
  'pnpm-lock.yaml',
  'package-lock.json',
  'yarn.lock',
  'bun.lockb',
  'tsconfig.json'
])

function formatTimeAgo(value?: string | null) {
  if (!value) return 'never'
  return formatDistanceToNow(new Date(value), { addSuffix: true })
}

function formatDateTime(value?: string | null) {
  if (!value) return '—'
  return new Date(value).toLocaleString()
}

function shortId(id: string) {
  return id.slice(0, 8)
}

function formatActor(actor: DeploymentRevision['actor']) {
  if (actor.username) return `@${actor.username}`
  return actor.user_id ?? actor.actor_type
}

function isDiffableFile(path: string) {
  const lowerPath = path.toLowerCase()
  const fileName = lowerPath.split('/').at(-1) ?? lowerPath
  if (DIFFABLE_FILENAMES.has(fileName)) return true
  return Array.from(DIFFABLE_EXTENSIONS).some((ext) => lowerPath.endsWith(ext))
}

function buildFileMap(files?: Array<{ path: string; contents: string }> | null) {
  const map = new Map<string, string>()
  for (const file of files ?? []) {
    map.set(file.path, file.contents)
  }
  return map
}

function toErrorMessage(error: unknown) {
  if (!(error instanceof Error)) return 'Request failed'
  return error.message
}

function statusBadgeClass(status: string) {
  if (status === 'failed') {
    return 'bg-rose-500/15 text-rose-700 hover:bg-rose-500/25 dark:bg-rose-500/10 dark:text-rose-300 dark:hover:bg-rose-500/20 border-0'
  }

  return 'bg-green-500/15 text-green-700 hover:bg-green-500/25 dark:bg-green-500/10 dark:text-green-300 dark:hover:bg-green-500/20 border-0'
}

function formatStatusLabel(status: string) {
  return status
    .split('_')
    .map((word) => (word ? word[0]?.toUpperCase() + word.slice(1) : word))
    .join(' ')
}

export function DeploymentHistory({ guildId }: { guildId: string }) {
  const [selectedRevisionId, setSelectedRevisionId] = useState<string | null>(null)
  const [diffOpen, setDiffOpen] = useState(false)
  const historyQuery = useDeploymentHistoryQuery(guildId)

  const selectedSummary = useMemo(
    () => historyQuery.data?.find((row) => row.id === selectedRevisionId),
    [historyQuery.data, selectedRevisionId]
  )

  useEffect(() => {
    if (!historyQuery.data?.length) {
      setSelectedRevisionId(null)
      return
    }

    if (!selectedRevisionId || !historyQuery.data.some((row) => row.id === selectedRevisionId)) {
      setSelectedRevisionId(historyQuery.data[0]?.id ?? null)
    }
  }, [historyQuery.data, selectedRevisionId])

  const shouldLoadDiffs = diffOpen && !!selectedRevisionId

  const selectedRevisionQuery = useDeploymentRevisionQuery(
    guildId,
    selectedRevisionId,
    shouldLoadDiffs
  )

  const selectedRevision = selectedRevisionQuery.data ?? selectedSummary ?? null
  const latestRevisionId = historyQuery.data?.[0]?.id ?? null
  const isLatestRevision = selectedRevision?.id === latestRevisionId
  const baseRevisionId = selectedRevision?.base_revision_id ?? null
  const baseRevisionQuery = useDeploymentRevisionQuery(
    guildId,
    baseRevisionId,
    diffOpen && !!baseRevisionId
  )

  const rollbackMutation = useRollbackDeploymentMutation(guildId, (revision) => {
    setSelectedRevisionId(revision.id)
  })

  const diffFiles = useMemo(() => {
    const current = buildFileMap(selectedRevisionQuery.data?.files)
    const base = buildFileMap(baseRevisionQuery.data?.files)

    const allPaths = new Set<string>([...current.keys(), ...base.keys()])

    return [...allPaths]
      .filter(isDiffableFile)
      .map((path) => ({
        path,
        oldContents: base.get(path) ?? '',
        newContents: current.get(path) ?? ''
      }))
      .filter((file) => file.oldContents !== file.newContents)
      .sort((a, b) => a.path.localeCompare(b.path))
  }, [baseRevisionQuery.data?.files, selectedRevisionQuery.data?.files])

  return (
    <div className='flex h-full min-h-0 flex-col gap-4'>
      <div className='rounded-lg border bg-card p-4'>
        {!selectedRevision
          ? (
            <div className='text-sm text-muted-foreground'>
              Select a revision to inspect metadata and source diffs.
            </div>
          )
          : shouldLoadDiffs && selectedRevisionQuery.isLoading
          ? (
            <div className='flex items-center gap-2 text-sm text-muted-foreground'>
              <Loader2 className='size-4 animate-spin' />
              Loading Revision Details…
            </div>
          )
          : selectedRevisionQuery.isError
          ? (
            <div className='text-sm text-destructive'>
              Failed to load revision: {toErrorMessage(selectedRevisionQuery.error)}
            </div>
          )
          : (
            <div className='space-y-3'>
              <div className='flex items-center justify-between gap-2'>
                <div>
                  <div className='flex items-center gap-2 text-sm font-medium'>
                    <Badge
                      variant='outline'
                      className={cn('border-0', statusBadgeClass(selectedRevision.status))}
                    >
                      {formatStatusLabel(selectedRevision.status)}
                    </Badge>
                    <span>Revision {selectedRevision.id}</span>
                  </div>
                  <div className='text-xs text-muted-foreground'>
                    {formatDateTime(selectedRevision.deployed_at)}
                  </div>
                </div>
                {!isLatestRevision
                  ? (
                    <Button
                      size='sm'
                      variant='outline'
                      disabled={rollbackMutation.isPending || selectedRevision.status !== 'success'}
                      onClick={() => {
                        if (selectedRevision.id) rollbackMutation.mutate(selectedRevision.id)
                      }}
                    >
                      {rollbackMutation.isPending
                        ? (
                          <>
                            <Loader2 className='mr-1 size-4 animate-spin' />
                            Rolling back…
                          </>
                        )
                        : (
                          <>
                            <RotateCcw className='mr-1 size-4' />
                            Rollback to this
                          </>
                        )}
                    </Button>
                  )
                  : null}
              </div>

              {rollbackMutation.isError
                ? (
                  <div className='text-xs text-destructive'>
                    Rollback failed: {toErrorMessage(rollbackMutation.error)}
                  </div>
                )
                : null}

              <div className='grid gap-3 md:grid-cols-2'>
                <div className='rounded-md border p-2'>
                  <div className='text-[11px] text-muted-foreground'>Source</div>
                  <div className='mt-1 text-sm font-medium'>{selectedRevision.deploy_source}</div>
                </div>
                <div className='rounded-md border p-2'>
                  <div className='text-[11px] text-muted-foreground'>Actor</div>
                  <div className='mt-1 text-sm font-medium'>
                    {formatActor(selectedRevision.actor)}
                  </div>
                </div>
              </div>

              <div className='grid grid-cols-2 gap-3 text-xs md:grid-cols-3'>
                <div className='rounded-md border p-2'>
                  <div className='text-muted-foreground'>Entry</div>
                  <div className='font-mono'>{selectedRevision.entry}</div>
                </div>
                <div className='rounded-md border p-2'>
                  <div className='text-muted-foreground'>Build</div>
                  <div className='font-mono'>{selectedRevision.build_id ?? '—'}</div>
                </div>
                <div className='rounded-md border p-2'>
                  <div className='text-muted-foreground'>Base Revision</div>
                  <div className='font-mono'>
                    {selectedRevision.base_revision_id
                      ? shortId(selectedRevision.base_revision_id)
                      : '—'}
                  </div>
                </div>
              </div>

              {selectedRevision.error_message
                ? (
                  <pre className='overflow-x-auto rounded border bg-muted/30 p-2 text-xs text-destructive'>
                    {selectedRevision.error_message}
                  </pre>
                )
                : null}
            </div>
          )}
      </div>

      <Collapsible open={diffOpen} onOpenChange={setDiffOpen} className='rounded-lg border bg-card'>
        <CollapsibleTrigger className='flex w-full items-center justify-between px-4 py-3 text-left'>
          <div>
            <div className='text-sm font-medium'>Source Diffs</div>
            <div className='text-xs text-muted-foreground'>{diffFiles.length} changed files</div>
          </div>
          <ChevronDown className={cn('size-4 transition-transform', diffOpen && 'rotate-180')} />
        </CollapsibleTrigger>
        <CollapsibleContent className='border-t p-3'>
          {shouldLoadDiffs && selectedRevisionQuery.isLoading
            ? <div className='text-sm text-muted-foreground'>Loading revision files…</div>
            : baseRevisionId && baseRevisionQuery.isLoading
            ? <div className='text-sm text-muted-foreground'>Loading base revision…</div>
            : baseRevisionQuery.isError
            ? (
              <div className='text-sm text-destructive'>
                Failed to load base revision: {toErrorMessage(baseRevisionQuery.error)}
              </div>
            )
            : !baseRevisionId
            ? <div className='text-sm text-muted-foreground'>No base revision for this entry.</div>
            : !diffFiles.length
            ? <div className='text-sm text-muted-foreground'>No diffable source-file changes.</div>
            : !diffOpen
            ? null
            : (
              <div className='max-h-[42dvh] space-y-3 overflow-auto'>
                {diffFiles.map((file) => (
                  <div key={file.path} className='overflow-hidden rounded-lg border'>
                    <Suspense
                      fallback={
                        <div className='flex items-center gap-2 px-3 py-2 text-xs text-muted-foreground'>
                          <Loader2 className='size-3 animate-spin' />
                          Loading diff renderer…
                        </div>
                      }
                    >
                      <LazyMultiFileDiff
                        oldFile={{ name: file.path, contents: file.oldContents }}
                        newFile={{ name: file.path, contents: file.newContents }}
                        options={{
                          diffStyle: 'split',
                          overflow: 'wrap',
                          lineDiffType: 'word'
                        }}
                      />
                    </Suspense>
                  </div>
                ))}
              </div>
            )}
        </CollapsibleContent>
      </Collapsible>

      <div className='min-h-0 flex-1 overflow-hidden rounded-lg border bg-card'>
        {historyQuery.isLoading
          ? (
            <div className='flex h-full items-center justify-center gap-2 text-sm text-muted-foreground'>
              <Loader2 className='size-4 animate-spin' />
              Loading Deployment History…
            </div>
          )
          : historyQuery.isError
          ? (
            <div className='flex h-full items-center justify-center text-sm text-destructive'>
              Failed to load history: {toErrorMessage(historyQuery.error)}
            </div>
          )
          : !historyQuery.data?.length
          ? (
            <div className='flex h-full items-center justify-center text-sm text-muted-foreground'>
              No deployments yet for this guild.
            </div>
          )
          : (
            <div className='h-full overflow-auto'>
              <Table>
                <TableHeader className='sticky top-0 bg-background/95 backdrop-blur'>
                  <TableRow className='hover:bg-transparent'>
                    <TableHead>Id</TableHead>
                    <TableHead>Actor</TableHead>
                    <TableHead>Deployed</TableHead>
                    <TableHead>Source</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Entry</TableHead>
                    <TableHead>Build</TableHead>
                    <TableHead>Error</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {historyQuery.data.map((row) => {
                    const isSelected = row.id === selectedRevisionId

                    return (
                      <TableRow
                        key={row.id}
                        data-state={isSelected ? 'selected' : undefined}
                        className='cursor-pointer'
                        onClick={() => {
                          setSelectedRevisionId(row.id)
                        }}
                      >
                        <TableCell className='font-mono text-xs'>{shortId(row.id)}</TableCell>
                        <TableCell className='text-xs'>{formatActor(row.actor)}</TableCell>
                        <TableCell className='text-xs whitespace-nowrap'>
                          {formatTimeAgo(row.deployed_at)}
                        </TableCell>
                        <TableCell className='text-xs'>{row.deploy_source}</TableCell>
                        <TableCell>
                          <Badge
                            variant='outline'
                            className={cn('border-0', statusBadgeClass(row.status))}
                          >
                            {row.status}
                          </Badge>
                        </TableCell>
                        <TableCell className='font-mono text-xs'>{row.entry}</TableCell>
                        <TableCell className='font-mono text-xs'>{row.build_id ?? '—'}</TableCell>
                        <TableCell className='max-w-60 text-xs'>
                          {row.error_message
                            ? (
                              <TooltipProvider>
                                <Tooltip>
                                  <TooltipTrigger>
                                    <span className='block truncate text-destructive'>
                                      {row.error_message}
                                    </span>
                                  </TooltipTrigger>
                                  <TooltipContent className='max-w-lg'>
                                    {row.error_message}
                                  </TooltipContent>
                                </Tooltip>
                              </TooltipProvider>
                            )
                            : '—'}
                        </TableCell>
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            </div>
          )}
      </div>
    </div>
  )
}
