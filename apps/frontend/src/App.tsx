import { Dashboard } from '@/components/dashboard'
import { DeploymentsPage } from '@/components/deployments-page'
import { EditorPage } from '@/components/editor-page'
import { OverviewPage } from '@/components/overview-page'
import { Settings } from '@/components/settings'
import { AppProvider } from '@/contexts/AppContext'
import { ThemeProvider } from '@/lib/theme'
import { Route, Switch } from 'wouter'

export default function App() {
  return (
    <ThemeProvider>
      <AppProvider>
        <Switch>
          <Route path='/' component={Dashboard} />
          <Route path='/:guildId' component={OverviewPage} />
          <Route path='/:guildId/editor' component={EditorPage} />
          <Route path='/:guildId/deployments' component={DeploymentsPage} />
          <Route path='/:guildId/settings' component={Settings} />
          <Route>404 - Not Found</Route>
        </Switch>
      </AppProvider>
    </ThemeProvider>
  )
}
