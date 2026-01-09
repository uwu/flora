import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/openapi-client";
import { DeploymentHistory } from "@/components/features/DeploymentHistory";
import { DashboardSidebar } from "@/components/sidebar-03/app-sidebar";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { useApp } from "@/contexts/AppContext";
import { useEffect } from "react";
import { useParams } from "wouter";

export function DeploymentsPage() {
  const { setView, setSelectedGuild } = useApp();
  const { guildId } = useParams<{ guildId: string }>();

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId);
    setView("deployments");
  }, [guildId, setSelectedGuild, setView]);

  const deploymentsQuery = useQuery({
    queryKey: ["deployments", guildId],
    enabled: !!guildId,
    queryFn: () =>
      api.GET("/deployments/{guild_id}", { params: { path: { guild_id: guildId! } } }).then((r) =>
        r.data ? [r.data] : []
      ),
  });

  return (
    <SidebarProvider>
      <div className="flex h-screen w-full bg-background text-foreground overflow-hidden font-sans">
        <DashboardSidebar />
        <SidebarInset className="flex min-w-0 flex-1 flex-col bg-background">
          <header className="sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/95 px-6 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <SidebarTrigger className="lg:hidden -ml-2" />
            <div className="font-medium">Deployments</div>
            <div className="ml-auto" />
          </header>
          <div className="flex-1 p-4 md:p-6 lg:p-8 overflow-y-auto">
            {deploymentsQuery.isLoading ? (
              <div className="text-sm text-muted-foreground">Loading deployments…</div>
            ) : deploymentsQuery.isError ? (
              <div className="text-sm text-destructive">Failed to load deployments</div>
            ) : (
              <DeploymentHistory deploymentsOverride={deploymentsQuery.data} />
            )}
          </div>
        </SidebarInset>
      </div>
    </SidebarProvider>
  );
}
