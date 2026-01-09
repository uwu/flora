import { useParams } from "wouter";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/openapi-client";
import { CodeEditor } from "@/components/features/CodeEditor";
import { DashboardSidebar } from "@/components/sidebar-03/app-sidebar";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import { redirectToLogin } from "@/lib/utils";
import { useState, useEffect } from "react";
import { useApp } from "@/contexts/AppContext";

export function EditorPage() {
  const { guildId } = useParams<{ guildId: string }>();
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [saveError, setSaveError] = useState<string | null>(null);
  const { setView, setSelectedGuild } = useApp();

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId);
    setView("editor");
  }, [guildId, setSelectedGuild, setView]);

  const deploymentQuery = useQuery({
    queryKey: ["deployment", guildId],
    enabled: !!guildId,
    queryFn: async () => {
      if (!guildId) return null;
      const res = await api.GET("/deployments/{guild_id}", { params: { path: { guild_id: guildId } } });
      if (res.error?.status === 401 || res.error?.status === 403) {
        redirectToLogin();
        return null;
      }
      return res.data ?? null;
    },
  });

  const handleSave = async (code: string) => {
    if (!guildId) return;
    setSaveStatus("saving");
    setSaveError(null);
    try {
      await api.POST("/deployments/{guild_id}", { params: { path: { guild_id: guildId } }, body: { code } });
      setSaveStatus("saved");
      deploymentQuery.refetch();
      setTimeout(() => setSaveStatus("idle"), 1500);
    } catch (err: any) {
      setSaveStatus("error");
      setSaveError(err.message || "Failed to save");
    }
  };

  const code = deploymentQuery.data?.source ?? "// Loading...";

  return (
    <SidebarProvider>
      <div className="flex h-screen w-full bg-background text-foreground overflow-hidden font-sans">
        <DashboardSidebar />
        <SidebarInset className="flex min-w-0 flex-1 flex-col bg-background">
          <div className="flex-1 min-h-0">
            {deploymentQuery.isLoading ? (
              <div className="text-sm text-muted-foreground">Loading code…</div>
            ) : deploymentQuery.isError ? (
              <div className="text-sm text-destructive">Failed to load code</div>
            ) : !deploymentQuery.data ? (
              <div className="text-sm text-muted-foreground">Login required.</div>
            ) : (
              <CodeEditor initialCode={code} guildId={guildId!} onSave={handleSave} />
            )}
          </div>
        </SidebarInset>
      </div>
    </SidebarProvider>
  );
}
