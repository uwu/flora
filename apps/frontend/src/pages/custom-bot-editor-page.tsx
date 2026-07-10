import { runEditorBuildFlow } from '@/components/editor/deploy-flow'
import { extractFilesFromDeployment } from '@/components/editor/editor-utils'
import { EditorWorkbench } from '@/components/editor/workbench'
import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { Button } from '@/components/ui/button'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { customBotScopeId, customBotsApi } from '@/data/custom-bots'
import { Seo } from '@/lib/seo'
import { useQuery } from '@tanstack/react-query'
import { ArrowLeft, Bot, Upload } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useLocation, useParams } from 'wouter'

const STARTER_FILES = {
  'src/main.ts': `createBot({
  prefix: '!',
  slashCommands: [
    slash({
      name: 'ping',
      description: 'Reply with pong',
      async run(ctx) {
        await ctx.reply('pong')
      }
    })
  ],
  commands: [
    prefix({
      name: 'ping',
      async run(ctx) {
        await ctx.reply('pong from a guild or DM')
      }
    })
  ]
})

console.log('custom bot loaded', customBot.id)
`
}

export function CustomBotEditorPage() {
  'use no memo'
  const { botId } = useParams<{ botId: string }>()
  const [, setLocation] = useLocation()
  const { setView, setSelectedGuild } = useApp()
  const [files, setFiles] = useState<Record<string, string>>({})
  const [deploying, setDeploying] = useState(false)
  const [status, setStatus] = useState<string | null>(null)
  const deployment = useQuery({
    queryKey: ['custom-bot-deployment', botId],
    queryFn: () => customBotsApi.deployment(botId ?? ''),
    enabled: !!botId,
    retry: false
  })
  const deployedFiles = useMemo(
    () => extractFilesFromDeployment(deployment.data),
    [deployment.data]
  )

  useEffect(() => {
    setSelectedGuild('')
    setView('custom-bot-editor')
  }, [setSelectedGuild, setView])

  useEffect(() => {
    if (Object.keys(deployedFiles).length > 0) {
      setFiles(deployedFiles)
      return
    }
    if (deployment.isError && deployment.error.message.toLowerCase().includes('not found')) {
      setFiles(STARTER_FILES)
    }
  }, [deployedFiles, deployment.error, deployment.isError])

  const deploy = async () => {
    if (!botId || deploying || Object.keys(files).length === 0) return
    setDeploying(true)
    setStatus('Building…')
    try {
      const result = await runEditorBuildFlow({
        guildId: customBotScopeId(botId),
        fileContents: files,
        preferredEntry: deployment.data?.entry ?? 'src/main.ts',
        fallbackEntry: 'src/main.ts',
        onBuildLog: (line) => setStatus(line)
      })
      await customBotsApi.deploy(botId, {
        entry: result.build.entry,
        files: Object.entries(files).map(([path, contents]) => ({ path, contents })),
        bundle: result.build.artifact.bundle,
        source_map: result.build.artifact.source_map
          ? { path: 'bundle.js.map', contents: result.build.artifact.source_map }
          : undefined
      })
      await deployment.refetch()
      setStatus('Deployed and running')
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Deploy failed')
    } finally {
      setDeploying(false)
    }
  }

  return (
    <>
      <Seo
        title='Custom bot editor'
        description='Edit and deploy a user-owned Flora bot.'
        path={`/bots/${botId}/editor`}
        noindex
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />
          <SidebarInset className='flex min-w-0 flex-1 flex-col'>
            <div className='absolute top-3 left-3 z-40 lg:hidden'>
              <SidebarTrigger />
            </div>
            <header className='flex h-14 shrink-0 items-center gap-3 border-b px-4 pl-14 lg:pl-4'>
              <Button variant='ghost' size='icon-sm' onClick={() => setLocation('/settings')}>
                <ArrowLeft />
              </Button>
              <Bot className='size-4' />
              <div className='min-w-0 flex-1'>
                <p className='truncate text-sm font-medium'>Custom bot editor</p>
                {status && <p className='truncate text-xs text-muted-foreground'>{status}</p>}
              </div>
              <Button size='sm' onClick={() => void deploy()} disabled={deploying || !botId}>
                <Upload /> {deploying ? 'Deploying…' : 'Deploy'}
              </Button>
            </header>
            <div className='min-h-0 flex-1'>
              {Object.keys(files).length > 0 ? (
                <EditorWorkbench
                  files={files}
                  entryFile={deployment.data?.entry ?? 'src/main.ts'}
                  runtime='custom-bot'
                  onFilesChange={setFiles}
                />
              ) : (
                <div className='flex h-full items-center justify-center text-sm text-muted-foreground'>
                  {deployment.isError ? deployment.error.message : 'Loading deployment…'}
                </div>
              )}
            </div>
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}
