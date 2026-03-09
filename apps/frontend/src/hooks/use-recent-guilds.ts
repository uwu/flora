import { useCallback, useEffect, useState } from 'react'

const RECENT_GUILDS_KEY = 'flora-recent-guilds'
const RECENT_GUILDS_EVENT = 'flora-recent-guilds-changed'

function readRecentGuildIds() {
  try {
    const stored = window.localStorage.getItem(RECENT_GUILDS_KEY)
    const parsed = stored ? (JSON.parse(stored) as string[]) : []
    return Array.isArray(parsed) ? parsed.filter((id): id is string => typeof id === 'string') : []
  } catch {
    return []
  }
}

function emitRecentGuildsChanged() {
  window.dispatchEvent(new CustomEvent(RECENT_GUILDS_EVENT))
}

export function useRecentGuilds(limit = 5) {
  const [recentGuildIds, setRecentGuildIds] = useState<string[]>([])

  useEffect(() => {
    setRecentGuildIds(readRecentGuildIds())

    const sync = () => setRecentGuildIds(readRecentGuildIds())

    window.addEventListener('storage', sync)
    window.addEventListener(RECENT_GUILDS_EVENT, sync)

    return () => {
      window.removeEventListener('storage', sync)
      window.removeEventListener(RECENT_GUILDS_EVENT, sync)
    }
  }, [])

  const pushRecentGuild = useCallback((guildId: string) => {
    const current = readRecentGuildIds()
    const next = [guildId, ...current.filter((id) => id !== guildId)].slice(0, limit)
    window.localStorage.setItem(RECENT_GUILDS_KEY, JSON.stringify(next))
    emitRecentGuildsChanged()
    setRecentGuildIds(next)
  }, [limit])

  const clearRecentGuilds = useCallback(() => {
    window.localStorage.removeItem(RECENT_GUILDS_KEY)
    emitRecentGuildsChanged()
    setRecentGuildIds([])
  }, [])

  return { recentGuildIds, pushRecentGuild, clearRecentGuilds }
}
