import { TokenManager } from '@/components/features/TokenManager'
import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
import { Server } from 'lucide-react'

import { LoginForm } from './login-page'

export function Dashboard() {
  const { session, sessionError, view } = useApp()

  if (sessionError) {
    return <FullScreenMessage />
  }

  if (!session) {
    return <FullScreenMessage />
  }

  return (
    <>
      <Seo
        title='Guild dashboard'
        description='Manage flora guild deployments, logs, and settings from one dashboard.'
        path='/'
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />

          <SidebarInset className='flex min-w-0 flex-1 flex-col'>
            {view === 'user-settings'
              ? (
                <div className='flex-1 overflow-y-auto p-4 md:p-6 lg:p-8'>
                  <div className='mx-auto max-w-4xl space-y-6'>
                    <div>
                      <h2 className='text-2xl font-bold tracking-tight'>User Settings</h2>
                      <p className='text-muted-foreground'>Manage your global account settings.</p>
                    </div>
                    <TokenManager />
                  </div>
                </div>
              )
              : (
                <div className='flex-1 p-4 md:p-6 lg:p-8'>
                  <div className='animate-in fade-in zoom-in-95 flex h-full flex-col items-center justify-center text-center duration-500'>
                    <div className='mb-4 rounded-full bg-primary/10 p-6'>
                      <Server className='h-10 w-10 text-primary' />
                    </div>
                    <h2 className='text-2xl font-bold tracking-tight'>No Guild Selected</h2>
                    <p className='mt-2 max-w-sm text-muted-foreground'>
                      Select a guild from the sidebar to manage its bot deployment, view logs, or
                      configure settings.
                    </p>
                  </div>
                </div>
              )}
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}

function FullScreenMessage() {
  return (
    <>
      <Seo
        title='Guild dashboard'
        description='Sign in with Discord to access the flora guild dashboard.'
        path='/'
      />
      <div className='flex min-h-svh flex-col items-center justify-center bg-muted p-6 md:p-10'>
        <div className='w-full max-w-sm md:max-w-4xl'>
          <LoginForm />
        </div>
      </div>
    </>
  )
}
