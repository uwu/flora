import { Dashboard } from '@/components/dashboard'
import { DeploymentsPage } from '@/components/deployments-page'
import { EditorPage } from '@/components/editor-page'
import { LoginPage } from '@/components/login-page'
import { OverviewPage } from '@/components/overview-page'
import { PrivacyPolicyPage } from '@/components/privacy-policy-page'
import { RequireAuth } from '@/components/require-auth'
import { Settings } from '@/components/settings'
import { TermsOfServicePage } from '@/components/terms-of-service-page'
import { AppProvider } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
import { ThemeProvider } from '@/lib/theme'
import type { ComponentType } from 'react'
import { Route, Switch } from 'wouter'

function NotFoundPage() {
  return (
    <div className='flex min-h-svh items-center justify-center p-6 text-sm text-muted-foreground'>
      <Seo title='Page not found' path='/404' noindex />
      404 - Not Found
    </div>
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
          <ProtectedRoute path='/' component={Dashboard} />
          <ProtectedRoute path='/:guildId/editor' component={EditorPage} />
          <ProtectedRoute path='/:guildId/deployments' component={DeploymentsPage} />
          <ProtectedRoute path='/:guildId/settings' component={Settings} />
          <ProtectedRoute path='/:guildId' component={OverviewPage} />
          <Route component={NotFoundPage} />
        </Switch>
      </AppProvider>
    </ThemeProvider>
  )
}
