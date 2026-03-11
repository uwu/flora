import { TokenManager } from '@/components/features/TokenManager'
import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { Seo } from '@/lib/seo'

export function UserSettingsPage() {
  return (
    <>
      <Seo
        title='User settings'
        description='Manage your flora account tokens and personal settings.'
        path='/settings'
        noindex
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />

          <SidebarInset className='flex min-w-0 flex-1 flex-col'>
            <div className='absolute top-3 left-3 z-40 lg:hidden'>
              <SidebarTrigger />
            </div>

            <div className='flex-1 overflow-y-auto p-4 md:p-6 lg:p-8'>
              <div className='mx-auto max-w-4xl space-y-6'>
                <div>
                  <h2 className='text-2xl font-bold tracking-tight'>User Settings</h2>
                  <p className='text-muted-foreground'>Manage your global account settings.</p>
                </div>
                <TokenManager />
              </div>
            </div>
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}
