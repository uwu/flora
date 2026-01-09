import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { api } from "@/lib/openapi-client";
import type { components } from "@/lib/openapi-schema";

type AuthUser = components["schemas"]["AuthUser"];
type Deployment = components["schemas"]["DeploymentResponse"];
type Guild = components["schemas"]["GuildResponse"];
type Token = components["schemas"]["TokenResponse"];

type LoadState<T> = {
  data: T | null;
  loading: boolean;
  error: string | null;
};

const initialState = { data: null, loading: true, error: null };

export type AppView = "guild" | "user-settings";

interface AppContextType {
  session: AuthUser | null;
  sessionError: string | null;
  guilds: LoadState<Guild[]>;
  deployments: LoadState<Deployment[]>;
  tokens: LoadState<Token[]>;
  selectedGuild: string;
  sidebarOpen: boolean;
  view: AppView;

  setSession: (session: AuthUser | null) => void;
  setSelectedGuild: (id: string) => void;
  setSidebarOpen: (open: boolean) => void;
  toggleSidebar: () => void;
  setView: (view: AppView) => void;

  refreshSession: () => Promise<void>;
  refreshGuilds: () => Promise<void>;
  refreshDeployments: () => Promise<void>;
  refreshTokens: () => Promise<void>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export function AppProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<AuthUser | null>(null);
  const [sessionError, setSessionError] = useState<string | null>(null);

  const [guilds, setGuilds] = useState<LoadState<Guild[]>>({ ...initialState });
  const [deployments, setDeployments] = useState<LoadState<Deployment[]>>({ ...initialState });
  const [tokens, setTokens] = useState<LoadState<Token[]>>({ ...initialState });

  const [selectedGuild, setSelectedGuild] = useState<string>("");
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [view, setView] = useState<AppView>("guild");

  const refreshSession = async () => {
    try {
      const res = await api.GET("/auth/me");
      setSession(res.data?.user ?? null);
      setSessionError(null);
    } catch (err: any) {
      if (err.status === 401) {
        setSession(null);
      } else {
        setSessionError(err.message || "Failed to load session");
      }
    }
  };

  const refreshGuilds = async () => {
    setGuilds((prev) => ({ ...prev, loading: true }));
    try {
      const res = await api.GET("/guilds");
      setGuilds({ data: res.data ?? null, loading: false, error: null });
    } catch (err: any) {
      setGuilds({ data: null, loading: false, error: err.message });
    }
  };

  const refreshDeployments = async () => {
    setDeployments((prev) => ({ ...prev, loading: true }));
    try {
      const res = await api.GET("/deployments");
      setDeployments({ data: res.data ?? null, loading: false, error: null });

      if (!selectedGuild && (res.data?.length ?? 0) > 0) {
        setSelectedGuild(res.data![0].guild_id);
      }
    } catch (err: any) {
      setDeployments({ data: null, loading: false, error: err.message });
    }
  };

  const refreshTokens = async () => {
    setTokens((prev) => ({ ...prev, loading: true }));
    try {
      const res = await api.GET("/tokens");
      setTokens({ data: res.data ?? null, loading: false, error: null });
    } catch (err: any) {
      setTokens({ data: null, loading: false, error: err.message });
    }
  };

  useEffect(() => {
    refreshSession();
  }, []);

  useEffect(() => {
    if (!session) return;

    setGuilds({ ...initialState });
    setDeployments({ ...initialState });
    setTokens({ ...initialState });

    refreshGuilds();
    refreshDeployments();
    refreshTokens();
  }, [session]);

  return (
    <AppContext.Provider
      value={{
        session,
        sessionError,
        guilds,
        deployments,
        tokens,
        selectedGuild,
        sidebarOpen,
        view,
        setSession,
        setSelectedGuild,
        setSidebarOpen,
        toggleSidebar: () => setSidebarOpen(!sidebarOpen),
        setView,
        refreshSession,
        refreshGuilds,
        refreshDeployments,
        refreshTokens,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error("useApp must be used within an AppProvider");
  }
  return context;
}
