import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
import { useEffect } from 'react'
import { useParams } from 'wouter'

export function OverviewPage() {
  const { guildId } = useParams<{ guildId: string }>()
  const { setView, setSelectedGuild } = useApp()

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId)
    setView('overview')
  }, [guildId, setSelectedGuild, setView])

  return (
    <>
      <Seo
        title='Guild overview'
        description='View flora guild status and high-level information.'
        path={guildId ? `/${guildId}` : '/'}
        noindex
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />
          <SidebarInset className='flex min-w-0 flex-1 flex-col'>
            <div className='absolute top-3 left-3 z-40 lg:hidden'>
              <SidebarTrigger />
            </div>
            <div className='flex flex-1 items-center justify-center text-muted-foreground'>
              <div className='space-y-2 text-center'>
                <div className='text-lg font-semibold'>Guild Overview</div>
                <div className='text-sm'>Guild ID: {guildId}</div>
                <div className='text-sm'>Coming soon.</div>
              </div>
            </div>
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}
