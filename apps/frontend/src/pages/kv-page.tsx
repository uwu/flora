import { KvManager } from '@/components/features/KvManager'
import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
import { useEffect } from 'react'
import { useParams } from 'wouter'

export function KvPage() {
  const { setView, setSelectedGuild } = useApp()
  const { guildId } = useParams<{ guildId: string }>()

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId)
    setView('kv')
  }, [guildId, setSelectedGuild, setView])

  return (
    <>
      <Seo
        title='Guild KV'
        description='Manage key-value stores and keys for this flora guild.'
        path={guildId ? `/${guildId}/kv` : '/kv'}
        noindex
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />
          <SidebarInset className='flex min-w-0 flex-1 flex-col'>
            <header className='supports-[backdrop-filter]:bg-background/60 sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/95 px-6 backdrop-blur'>
              <SidebarTrigger className='-ml-2 lg:hidden' />
              <div className='font-medium'>KV</div>
              <div className='ml-auto' />
            </header>
            <div className='flex-1 overflow-y-auto p-4 md:p-6 lg:p-8'>
              {!guildId ? (
                <div className='text-sm text-destructive'>Missing guild id</div>
              ) : (
                <KvManager guildId={guildId} />
              )}
            </div>
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}
