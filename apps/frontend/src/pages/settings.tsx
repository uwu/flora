import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
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
    <>
      <Seo
        title='Guild settings'
        description='Adjust flora settings for this guild.'
        path={guildId ? `/${guildId}/settings` : '/settings'}
        noindex
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />
          <SidebarInset className='flex min-w-0 flex-1 flex-col'>
            <header className='supports-[backdrop-filter]:bg-background/60 sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/95 px-6 backdrop-blur'>
              <SidebarTrigger className='-ml-2 lg:hidden' />
              <div className='font-medium'>Settings</div>
              <div className='ml-auto' />
            </header>
            <div className='flex-1 overflow-y-auto p-4 md:p-6 lg:p-8'>
              <div className='text-sm text-muted-foreground'>
                Settings for guild {guildId ?? 'unknown'} coming soon.
              </div>
            </div>
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}
