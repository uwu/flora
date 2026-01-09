import { Route, Switch } from "wouter";
import { Dashboard } from "@/components/dashboard";
import { AppProvider } from "@/contexts/AppContext";
import { ThemeProvider } from "@/lib/theme";
import { Settings } from "@/components/settings";
import { DeploymentsPage } from "@/components/deployments-page";
import { EditorPage } from "@/components/editor-page";
import { OverviewPage } from "@/components/overview-page";

export function App() {
  return (
    <ThemeProvider>
      <AppProvider>
        <Switch>
          <Route path="/" component={Dashboard} />
          <Route path="/:guildId" component={OverviewPage} />
          <Route path="/:guildId/editor" component={EditorPage} />
          <Route path="/:guildId/deployments" component={DeploymentsPage} />
          <Route path="/:guildId/settings" component={Settings} />
          <Route>404 - Not Found</Route>
        </Switch>
      </AppProvider>
    </ThemeProvider>
  );
}

export default App;
