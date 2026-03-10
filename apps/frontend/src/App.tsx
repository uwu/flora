import { RequireAuth } from '@/components/require-auth'
import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { AppProvider, useApp } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
import { ThemeProvider } from '@/lib/theme'
import { Dashboard } from '@/pages/dashboard'
import { DeploymentsPage } from '@/pages/deployments-page'
import { KvPage } from '@/pages/kv-page'
import { LoginPage } from '@/pages/login-page'
import { OverviewPage } from '@/pages/overview-page'
import { PrivacyPolicyPage } from '@/pages/privacy-policy-page'
import { Settings } from '@/pages/settings'
import { TermsOfServicePage } from '@/pages/terms-of-service-page'
import { UserSettingsPage } from '@/pages/user-settings-page'
import { type ComponentType, lazy, Suspense, useEffect } from 'react'
import { Route, Switch, useParams } from 'wouter'

function NotFoundPage() {
  return (
    <div className='flex min-h-svh items-center justify-center p-6 text-sm text-muted-foreground'>
      <Seo title='Page not found' path='/404' noindex />
      404 - Not Found
    </div>
  )
}

const LazyEditorPage = lazy(async () => {
  const module = await import('@/pages/editor-page')
  return { default: module.EditorPage }
})

function EditorPageLoadingFallback() {
  const { guildId } = useParams<{ guildId: string }>()
  const { setView, setSelectedGuild } = useApp()

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId)
    setView('editor')
  }, [guildId, setSelectedGuild, setView])

  return (
    <SidebarProvider>
      <div className='relative flex h-dvh w-full'>
        <DashboardSidebar />
        <SidebarInset className='flex min-w-0 flex-1'>
          <div className='absolute top-3 left-3 z-40 lg:hidden'>
            <SidebarTrigger />
          </div>
          <div className='flex h-full min-h-0 w-full items-center justify-center text-sm text-muted-foreground'>
            Loading editor…
          </div>
        </SidebarInset>
      </div>
    </SidebarProvider>
  )
}

function EditorPageRoute() {
  return (
    <Suspense fallback={<EditorPageLoadingFallback />}>
      <LazyEditorPage />
    </Suspense>
  )
}

function ProtectedRoute({
  path,
  component: Component
}: {
  path: string
  component: ComponentType
}) {
  return (
    <Route path={path}>
      <RequireAuth>
        <Component />
      </RequireAuth>
    </Route>
  )
}

export default function App() {
  return (
    <ThemeProvider>
      <AppProvider>
        <Switch>
          <Route path='/login' component={LoginPage} />
          <Route path='/terms-of-service' component={TermsOfServicePage} />
          <Route path='/privacy-policy' component={PrivacyPolicyPage} />
          <ProtectedRoute path='/settings' component={UserSettingsPage} />
          <ProtectedRoute path='/' component={Dashboard} />
          <ProtectedRoute path='/:guildId/editor' component={EditorPageRoute} />
          <ProtectedRoute path='/:guildId/deployments' component={DeploymentsPage} />
          <ProtectedRoute path='/:guildId/kv' component={KvPage} />
          <ProtectedRoute path='/:guildId/settings' component={Settings} />
          <ProtectedRoute path='/:guildId' component={OverviewPage} />
          <Route component={NotFoundPage} />
        </Switch>
      </AppProvider>
    </ThemeProvider>
  )
}
