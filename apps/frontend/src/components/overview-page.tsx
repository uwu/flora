import { DashboardSidebar } from '@/components/sidebar-03/app-sidebar'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
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
    <SidebarProvider>
      <div className='flex h-screen w-full bg-background text-foreground overflow-hidden font-sans'>
        <DashboardSidebar />
        <SidebarInset className='flex min-w-0 flex-1 flex-col bg-background'>
          <div className='flex-1 flex items-center justify-center text-muted-foreground'>
            <div className='text-center space-y-2'>
              <div className='text-lg font-semibold'>Guild Overview</div>
              <div className='text-sm'>Guild ID: {guildId}</div>
              <div className='text-sm'>Coming soon.</div>
            </div>
          </div>
        </SidebarInset>
      </div>
    </SidebarProvider>
  )
}
