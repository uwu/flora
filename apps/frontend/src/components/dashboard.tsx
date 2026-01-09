import { useEffect, useMemo, useState } from "react";
import { redirectToLogin } from "@/lib/utils";
import { api } from "@/lib/openapi-client";
import type { components } from "@/lib/openapi-schema";
import { TokenManager } from "@/components/features/TokenManager";
import { DashboardSidebar } from "@/components/sidebar-03/app-sidebar";
import { SidebarProvider, SidebarInset, SidebarTrigger } from "@/components/ui/sidebar";

import { formatDistanceToNow } from "date-fns";
import {
  LayoutDashboard,
  Code2,
  History,
  Moon,
  Sun,
  Play,
  Clock,
  CheckCircle2,
  XCircle,
  Server,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

import Monaco from "@uwu/monaco-react";
import { Badge } from "@/components/ui/badge";
import { ScrollArea, ScrollBar } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { useTheme } from "@/lib/theme";
import { useApp } from "@/contexts/AppContext";

function formatTimeAgo(value?: string | null) {
  if (!value) return "never";
  return formatDistanceToNow(new Date(value), { addSuffix: true });
}

type LoadState<T> = {
  data: T | null;
  loading: boolean;
  error: string | null;
};

const initialState = { data: null, loading: true, error: null };

type Tab = "overview" | "editor" | "deployments";

export function Dashboard() {
  const [session, setSession] = useState<components["schemas"]["AuthUser"] | null>(null);
  const [sessionError, setSessionError] = useState<string | null>(null);

  const [guilds, setGuilds] = useState<LoadState<components["schemas"]["GuildResponse"][]>>({ ...initialState });
  const [deployments, setDeployments] = useState<
    LoadState<components["schemas"]["DeploymentResponse"][]>
  >({ ...initialState });

  const [selectedGuild, setSelectedGuild] = useState<string>("");
  const [activeTab, setActiveTab] = useState<Tab>("editor");

  const [code, setCode] = useState<string>("");
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [saveError, setSaveError] = useState<string | null>(null);
  const [isCodeLoading, setIsCodeLoading] = useState(false);

  const { theme, toggleTheme } = useTheme();
  const { view } = useApp();

  useEffect(() => {
    let mounted = true;
    api
      .GET("/auth/me")
      .then((res) => {
        if (!mounted) return;
        setSession(res.data?.user ?? null);
        setSessionError(null);
      })
      .catch((err) => {
        if (!mounted) return;
        if (err.status === 401) {
          setSession(null);
        } else {
          setSessionError(err.message || "Failed to load session");
        }
      });
    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    if (!session) return;

    setGuilds({ ...initialState });
    api
      .GET("/guilds")
      .then((res) => setGuilds({ data: res.data ?? null, loading: false, error: null }))
      .catch((err) => setGuilds({ data: null, loading: false, error: err.message }));

    setDeployments({ ...initialState });
    api
      .GET("/deployments")
      .then((res) => {
        const data = res.data ?? [];
        setDeployments({ data, loading: false, error: null });
        if (!selectedGuild && data.length > 0) {
          setSelectedGuild(data[0].guild_id);
        }
      })
      .catch((err) => setDeployments({ data: null, loading: false, error: err.message }));
  }, [session]);

  useEffect(() => {
    if (!selectedGuild) return;

    setIsCodeLoading(true);
    api
      .GET("/deployments/{guild_id}", { params: { path: { guild_id: selectedGuild } } })
      .then((dep) => {
        if (dep.data?.source) {
          setCode(dep.data.source);
        } else {
          setCode(getDefaultCode());
        }
      })
      .catch(() => {
        setCode(getDefaultCode());
      })
      .finally(() => setIsCodeLoading(false));
  }, [selectedGuild]);

  const getDefaultCode = () =>
    "// Write your guild bot here\nexport default async function main(ctx) {\n  ctx.reply('Hello from flora!')\n}\n";

  const handleSave = async () => {
    if (!selectedGuild) return;
    setSaveStatus("saving");
    setSaveError(null);
    try {
      await api.POST("/deployments/{guild_id}", {
        params: { path: { guild_id: selectedGuild } },
        body: { code },
      });
      setSaveStatus("saved");
      const refreshed = await api.GET("/deployments");
      setDeployments({ data: refreshed.data ?? null, loading: false, error: null });
      setTimeout(() => setSaveStatus("idle"), 2000);
    } catch (err: any) {
      setSaveStatus("error");
      setSaveError(err.message || "Failed to save");
    }
  };

  const selectedGuildInfo = useMemo(
    () => guilds.data?.find((g) => g.id === selectedGuild),
    [guilds.data, selectedGuild],
  );

  if (sessionError) {
    return (
      <FullScreenMessage
        icon={XCircle}
        title="Connection Failed"
        description={sessionError}
        actionLabel="Retry"
        onAction={() => window.location.reload()}
      />
    );
  }

  if (!session) {
    return (
      <FullScreenMessage
        icon={LayoutDashboard}
        title="Welcome to flora"
        description="Sign in with Discord to manage your guild deployments."
        actionLabel="Sign in with Discord"
        onAction={redirectToLogin}
      />
    );
  }

  return (
    <SidebarProvider>
      <div className="flex h-screen w-full bg-background text-foreground overflow-hidden font-sans">
        <DashboardSidebar />

        <SidebarInset className="flex min-w-0 flex-1 flex-col bg-background">
          <header className="sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/95 px-6 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <SidebarTrigger className="lg:hidden -ml-2" />

            <div className="flex items-center gap-2">
              <span className="text-muted-foreground hidden sm:inline-block">/</span>
              <span className="font-medium">
                {view === "user-settings"
                  ? "User Settings"
                  : selectedGuildInfo
                    ? selectedGuildInfo.name
                    : "Select a Guild"}
              </span>
            </div>

            <div className="ml-auto flex items-center gap-2">
              <Button variant="ghost" size="icon" onClick={toggleTheme}>
                {theme === "dark" ? <Moon className="h-4 w-4" /> : <Sun className="h-4 w-4" />}
              </Button>
            </div>
          </header>

          {view === "user-settings" ? (
            <div className="flex-1 p-4 md:p-6 lg:p-8 overflow-y-auto">
              <div className="max-w-4xl mx-auto space-y-6">
                <div>
                  <h2 className="text-2xl font-bold tracking-tight">User Settings</h2>
                  <p className="text-muted-foreground">Manage your global account settings.</p>
                </div>
                <TokenManager />
              </div>
            </div>
          ) : (
            <>
              {selectedGuild ? (
                <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as Tab)} className="flex-1 flex flex-col">
                    <ScrollArea>
                      <TabsList className="mb-3">
                        <TabsTrigger value="editor">
                          <Code2
                            aria-hidden="true"
                            className="-ms-0.5 me-1.5 opacity-60"
                            size={16}
                          />
                          Editor
                        </TabsTrigger>
                        <TabsTrigger value="deployments">
                          <History
                            aria-hidden="true"
                            className="-ms-0.5 me-1.5 opacity-60"
                            size={16}
                          />
                          Deployments
                        </TabsTrigger>
                      </TabsList>
                      <ScrollBar orientation="horizontal" />
                    </ScrollArea>

                    <TabsContent value="editor" className="flex-1 flex flex-col min-h-0">
                      <div className="flex flex-col flex-1 min-h-0 mx-auto w-full animate-in fade-in slide-in-from-bottom-4 duration-500 max-w-6xl">
                        <div className="flex items-center justify-end px-4 py-2 bg-background border-b">
                          <div className="flex items-center gap-2">
                            {saveStatus === "error" && saveError && (
                              <span className="text-xs text-destructive flex items-center gap-1 animate-in fade-in">
                                <XCircle className="h-3 w-3" /> {saveError}
                              </span>
                            )}
                            {saveStatus === "saved" && (
                              <span className="text-xs text-green-600 flex items-center gap-1 animate-in fade-in">
                                <CheckCircle2 className="h-3 w-3" /> Saved
                              </span>
                            )}
                            <Button
                              size="sm"
                              onClick={handleSave}
                              disabled={saveStatus === "saving" || isCodeLoading}
                              className={cn(
                                "transition-all",
                                saveStatus === "saved" ? "bg-green-600 hover:bg-green-700" : "",
                              )}
                            >
                              {saveStatus === "saving" ? (
                                <Clock className="mr-2 h-3 w-3 animate-spin" />
                              ) : (
                                <Play className="mr-2 h-3 w-3 fill-current" />
                              )}
                              {saveStatus === "saved" ? "Deployed" : "Deploy"}
                            </Button>
                          </div>
                        </div>
                        <div className="relative flex-1 bg-zinc-950 h-full w-full">
                          {isCodeLoading && (
                            <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/50 backdrop-blur-sm">
                              <Clock className="h-8 w-8 animate-spin text-primary" />
                            </div>
                          )}
                          <Monaco
                            value={code}
                            valOut={setCode}
                            lang="typescript"
                            theme="vs-dark"
                            height="100%"
                            width="100%"
                            readonly={isCodeLoading}
                            otherCfg={{
                              minimap: { enabled: false },
                              fontSize: 14,
                              lineNumbers: "on",
                              scrollBeyondLastLine: false,
                              automaticLayout: true,
                              padding: { top: 16, bottom: 16 },
                            }}
                          />
                        </div>
                      </div>
                    </TabsContent>

                    <TabsContent value="deployments" className="flex-1 overflow-y-auto">
                      <div className="mx-auto w-full animate-in fade-in slide-in-from-bottom-4 duration-500 max-w-6xl space-y-6">
                        <Card>
                          <CardHeader>
                            <CardTitle>Deployment History</CardTitle>
                            <CardDescription>Recent updates to your guild bots.</CardDescription>
                          </CardHeader>
                          <CardContent>
                            {!deployments.data?.length ? (
                              <EmptyState
                                icon={History}
                                title="No deployments"
                                description="You haven't deployed any code yet."
                              />
                            ) : (
                              <div className="space-y-4">
                                {deployments.data
                                  .filter((d) => d.guild_id === selectedGuild || !selectedGuild)
                                  .map((dep) => (
                                    <div
                                      key={dep.guild_id}
                                      className="flex items-center justify-between rounded-lg border p-4 transition-colors hover:bg-muted/50"
                                    >
                                      <div className="flex items-center gap-4">
                                        <div className="rounded-full bg-primary/10 p-2 text-primary">
                                          <Code2 className="h-4 w-4" />
                                        </div>
                                        <div>
                                          <p className="font-medium text-sm">
                                            {guilds.data?.find((g) => g.id === dep.guild_id)?.name ||
                                              dep.guild_id}
                                          </p>
                                          <p className="text-xs text-muted-foreground">
                                            Deployed {formatTimeAgo(dep.updated_at)}
                                          </p>
                                        </div>
                                      </div>
                                      <div className="flex items-center gap-3">
                                        <Badge variant="secondary" className="font-mono text-xs">
                                          typescript
                                        </Badge>
                                      </div>
                                    </div>
                                  ))}
                              </div>
                            )}
                          </CardContent>
                        </Card>
                      </div>
                    </TabsContent>
                </Tabs>
              ) : (
                <div className="flex-1 p-4 md:p-6 lg:p-8">
                  <div className="flex h-full flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500">
                    <div className="rounded-full bg-primary/10 p-6 mb-4">
                      <Server className="h-10 w-10 text-primary" />
                    </div>
                    <h2 className="text-2xl font-bold tracking-tight">No Guild Selected</h2>
                    <p className="text-muted-foreground mt-2 max-w-sm">
                      Select a guild from the sidebar to manage its bot deployment, view history, or
                      configure settings.
                    </p>
                  </div>
                </div>
              )}
            </>
          )}
        </SidebarInset>
      </div>
    </SidebarProvider>
  );
}



function EmptyState({
  icon: Icon,
  title,
  description,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  description: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-8 text-center animate-in fade-in zoom-in-95">
      <div className="rounded-full bg-muted p-3">
        <Icon className="h-6 w-6 text-muted-foreground" />
      </div>
      <div className="font-medium mt-2">{title}</div>
      <div className="text-muted-foreground text-sm max-w-xs">{description}</div>
    </div>
  );
}

function FullScreenMessage({
  icon: Icon,
  title,
  description,
  actionLabel,
  onAction,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  description: string;
  actionLabel: string;
  onAction: () => void;
}) {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background p-4 text-foreground">
      <div className="mx-auto flex max-w-[400px] flex-col items-center justify-center text-center space-y-6">
        <div className="flex h-20 w-20 items-center justify-center rounded-3xl bg-primary/10 text-primary ring-8 ring-primary/5">
          <Icon className="h-10 w-10" />
        </div>
        <div className="space-y-2">
          <h1 className="text-2xl font-bold tracking-tight">{title}</h1>
          <p className="text-muted-foreground">{description}</p>
        </div>
        <Button onClick={onAction} size="lg" className="w-full">
          {actionLabel}
        </Button>
      </div>
    </div>
  );
}
