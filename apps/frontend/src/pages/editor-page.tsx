import { runEditorBuildFlow } from '@/components/editor/deploy-flow'
import { EditorSidePanel } from '@/components/editor/editor-side-panel'
import { extractFilesFromDeployment } from '@/components/editor/editor-utils'
import { EditorWorkbench } from '@/components/editor/workbench'
import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { useDeployMutation } from '@/data/mutations'
import { useDeploymentQuery, useLogsQuery } from '@/data/queries'
import { Seo } from '@/lib/seo'
import { cn } from '@/lib/utils'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'wouter'

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

export function EditorPage() {
  'use no memo'
  const { guildId } = useParams<{ guildId: string }>()
  const { setView, setSelectedGuild } = useApp()
  const [logsSectionOpen, setLogsSectionOpen] = useState(true)
  const [fileContents, setFileContents] = useState<Record<string, string>>({})
  const [isDeploying, setIsDeploying] = useState(false)
  const [deployState, setDeployState] = useState<'idle' | 'deploying' | 'success' | 'error'>('idle')
  const [deployError, setDeployError] = useState<string | null>(null)
  const [deployBuildLogs, setDeployBuildLogs] = useState<string[]>([])
  const [deployUploadedFiles, setDeployUploadedFiles] = useState<string[]>([])
  const [copyState, setCopyState] = useState<'idle' | 'copied'>('idle')
  const [tailLogs, setTailLogs] = useState(false)
  const logsAreaRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId)
    setView('editor')
  }, [guildId, setSelectedGuild, setView])

  const deploymentQuery = useDeploymentQuery(guildId)

  const logsQuery = useLogsQuery(guildId)

  const deployMutation = useDeployMutation()

  const filesFromDeployment = useMemo(
    () => extractFilesFromDeployment(deploymentQuery.data),
    [deploymentQuery.data]
  )

  useEffect(() => {
    if (Object.keys(filesFromDeployment).length === 0) return

    const timer = window.setTimeout(() => {
      setFileContents(filesFromDeployment)
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [filesFromDeployment])

  const hasUnsavedChanges = useMemo(
    () => !areFileMapsEqual(fileContents, filesFromDeployment),
    [fileContents, filesFromDeployment]
  )

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
    if (deployState !== 'success') return
    const timer = window.setTimeout(() => setDeployState('idle'), 1800)
    return () => window.clearTimeout(timer)
  }, [deployState])

  const handleDeploy = () => {
    if (!guildId || isDeploying) return
    setDeployError(null)
    setDeployState('deploying')
    setCopyState('idle')
    setIsDeploying(true)
    setDeployBuildLogs([])

    const preferredEntry = deploymentQuery.data?.entry ?? 'src/main.ts'
    const fallbackEntry = preferredEntry ?? Object.keys(fileContents)[0] ?? 'src/main.ts'

    void runEditorBuildFlow({
      guildId,
      fileContents,
      preferredEntry,
      fallbackEntry,
      onBuildLog: (line) => setDeployBuildLogs((prev) => [...prev, line])
    })
      .then(async (buildResult) => {
        setDeployUploadedFiles(buildResult.uploadedFiles)

        const deployFiles = Object.entries(fileContents)
          .map(([path, contents]) => ({ path, contents }))
          .sort((a, b) => a.path.localeCompare(b.path))

        await deployMutation.mutateAsync({
          params: { path: { guild_id: buildResult.build.guild_id } },
          headers: { 'x-flora-deploy-source': 'webui' },
          body: {
            entry: buildResult.build.entry,
            files: deployFiles,
            bundle: buildResult.build.artifact.bundle
          }
        })

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

  const entryCandidates = [
    deploymentQuery.data?.entry,
    deploymentQuery.data?.entry ? `src/${deploymentQuery.data.entry}` : null,
    Object.keys(fileContents)[0]
  ].filter((value): value is string => !!value)
  const entryFile = entryCandidates.find((candidate) => fileContents[candidate] != null)

  return (
    <>
      <Seo
        title='Guild editor'
        description='Edit and deploy bot code for this flora guild.'
        path={guildId ? `/${guildId}/editor` : '/editor'}
        noindex
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />
          <SidebarInset className='flex min-w-0 flex-1'>
            <div className='absolute top-3 left-3 z-40 lg:hidden'>
              <SidebarTrigger />
            </div>
            <div className='flex h-full min-h-0 w-full overflow-hidden'>
              <EditorWorkbench
                files={fileContents}
                entryFile={entryFile}
                onFilesChange={setFileContents}
              />
              <EditorSidePanel
                logsSectionOpen={logsSectionOpen}
                onLogsSectionOpenChange={setLogsSectionOpen}
                deployError={deployError}
                deployButtonClass={deployButtonClass}
                deployLabel={deployLabel}
                deployUploadedFiles={deployUploadedFiles}
                deployBuildLogs={deployBuildLogs}
                copyState={copyState}
                guildId={guildId}
                fileCount={Object.keys(fileContents).length}
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
            </div>
            {hasUnsavedChanges && (
              <div className='pointer-events-none absolute right-4 bottom-4 rounded-full border border-amber-500/40 bg-amber-500/10 px-3 py-1 text-[10px] font-semibold text-amber-600'>
                Unsaved changes
              </div>
            )}
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}
