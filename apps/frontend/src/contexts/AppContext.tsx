import { api } from '@/lib/openapi-client'
import type { components } from '@/lib/openapi-schema'
import { createContext, type ReactNode, useCallback, useContext, useEffect, useState } from 'react'

type AuthUser = components['schemas']['AuthUser']
type Deployment = components['schemas']['DeploymentResponse']
type Guild = components['schemas']['GuildResponse']
type Token = components['schemas']['TokenResponse']

type LoadState<T> = {
  data: T | null
  loading: boolean
  error: string | null
}

const initialState = { data: null, loading: true, error: null }

type AppView = 'guild' | 'overview' | 'editor' | 'deployments' | 'user-settings'

interface AppContextType {
  session: AuthUser | null
  sessionLoading: boolean
  sessionError: string | null
  guilds: LoadState<Guild[]>
  deployments: LoadState<Deployment[]>
  tokens: LoadState<Token[]>
  selectedGuild: string
  sidebarOpen: boolean
  view: AppView

  setSession: (session: AuthUser | null) => void
  setSelectedGuild: (id: string) => void
  setSidebarOpen: (open: boolean) => void
  toggleSidebar: () => void
  setView: (view: AppView) => void

  refreshSession: () => Promise<void>
  refreshGuilds: () => Promise<void>
  refreshDeployments: () => Promise<void>
  refreshTokens: () => Promise<void>
}

const AppContext = createContext<AppContextType | undefined>(undefined)

export function AppProvider({ children }: { children: ReactNode }) {
  const [session, setSession] = useState<AuthUser | null>(null)
  const [sessionLoading, setSessionLoading] = useState(true)
  const [sessionError, setSessionError] = useState<string | null>(null)

  const [guilds, setGuilds] = useState<LoadState<Guild[]>>({ ...initialState })
  const [deployments, setDeployments] = useState<LoadState<Deployment[]>>({ ...initialState })
  const [tokens, setTokens] = useState<LoadState<Token[]>>({ ...initialState })

  const [selectedGuild, setSelectedGuild] = useState<string>('')
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [view, setView] = useState<AppView>('guild')

  const refreshSession = useCallback((): Promise<void> => {
    setSessionLoading(true)

    return api
      .GET('/auth/me', {})
      .then((res) => {
        const user = res.data ? res.data.user : null
        setSession(user)
        setSessionError(null)
      })
      .catch((err: any) => {
        if (err.status === 401) {
          setSession(null)
          setSessionError(null)
        } else {
          setSessionError(err.message || 'Failed to load session')
        }
      })
      .finally(() => {
        setSessionLoading(false)
      })
  }, [])

  const refreshGuilds = useCallback((): Promise<void> => {
    return api
      .GET('/guilds/', {})
      .then((res) => {
        setGuilds({ data: res.data ?? null, loading: false, error: null })
      })
      .catch((err: any) => {
        setGuilds({ data: null, loading: false, error: err.message })
      })
  }, [])

  const refreshDeployments = useCallback((): Promise<void> => {
    return api
      .GET('/deployments/', {})
      .then((res) => {
        setDeployments({ data: res.data ?? null, loading: false, error: null })

        if ((res.data?.length ?? 0) > 0) {
          const firstGuildId = res.data![0].guild_id
          setSelectedGuild((prev) => prev || firstGuildId)
        }
      })
      .catch((err: any) => {
        setDeployments({ data: null, loading: false, error: err.message })
      })
  }, [])

  const refreshTokens = useCallback((): Promise<void> => {
    return api
      .GET('/tokens/', {})
      .then((res) => {
        setTokens({ data: res.data ?? null, loading: false, error: null })
      })
      .catch((err: any) => {
        setTokens({ data: null, loading: false, error: err.message })
      })
  }, [])

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void refreshSession()
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [refreshSession])

  useEffect(() => {
    if (!session) return

    const timer = window.setTimeout(() => {
      void refreshGuilds()
      void refreshDeployments()
      void refreshTokens()
    }, 0)

    return () => {
      window.clearTimeout(timer)
    }
  }, [refreshDeployments, refreshGuilds, refreshTokens, session])

  return (
    <AppContext.Provider
      value={{
        session,
        sessionLoading,
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
        toggleSidebar: () => setSidebarOpen((prev) => !prev),
        setView,
        refreshSession,
        refreshGuilds,
        refreshDeployments,
        refreshTokens
      }}
    >
      {children}
    </AppContext.Provider>
  )
}

export function useApp() {
  const context = useContext(AppContext)
  if (context === undefined) {
    throw new Error('useApp must be used within an AppProvider')
  }
  return context
}
