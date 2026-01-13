import { DashboardSidebar } from '@/components/sidebar-03/app-sidebar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { useEffect } from 'react'
import { useParams } from 'wouter'

export function Settings() {
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
          <header className='sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/95 px-6 backdrop-blur supports-[backdrop-filter]:bg-background/60'>
            <SidebarTrigger className='lg:hidden -ml-2' />
            <div className='font-medium'>Settings</div>
            <div className='ml-auto' />
          </header>
          <div className='flex-1 p-4 md:p-6 lg:p-8 overflow-y-auto'>
            <div className='text-sm text-muted-foreground'>
              Settings for guild {guildId ?? 'unknown'} coming soon.
            </div>
          </div>
        </SidebarInset>
      </div>
    </SidebarProvider>
  )
}
