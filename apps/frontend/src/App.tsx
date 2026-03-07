import { Dashboard } from '@/components/dashboard'
import { DeploymentsPage } from '@/components/deployments-page'
import { EditorPage } from '@/components/editor-page'
import { OverviewPage } from '@/components/overview-page'
import { PrivacyPolicyPage } from '@/components/privacy-policy-page'
import { Settings } from '@/components/settings'
import { TermsOfServicePage } from '@/components/terms-of-service-page'
import { AppProvider } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
import { ThemeProvider } from '@/lib/theme'
import { Route, Switch } from 'wouter'

function NotFoundPage() {
  return (
    <div className='flex min-h-svh items-center justify-center p-6 text-sm text-muted-foreground'>
      <Seo title='Page not found' path='/404' noindex />
      404 - Not Found
    </div>
  )
}

export default function App() {
  return (
    <ThemeProvider>
      <AppProvider>
        <Switch>
          <Route path='/' component={Dashboard} />
          <Route path='/terms-of-service' component={TermsOfServicePage} />
          <Route path='/privacy-policy' component={PrivacyPolicyPage} />
          <Route path='/:guildId/editor' component={EditorPage} />
          <Route path='/:guildId/deployments' component={DeploymentsPage} />
          <Route path='/:guildId/settings' component={Settings} />
          <Route path='/:guildId' component={OverviewPage} />
          <Route component={NotFoundPage} />
        </Switch>
      </AppProvider>
    </ThemeProvider>
  )
}
