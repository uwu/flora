import { useApp } from '@/contexts/AppContext'
import { Seo } from '@/lib/seo'
import { type ReactNode, useEffect } from 'react'
import { useLocation, useSearch } from 'wouter'

function toSafeNext(next: string | null) {
  if (!next) return '/'
  if (!next.startsWith('/')) return '/'
  if (next.startsWith('//')) return '/'
  return next
}

export function RequireAuth({ children }: { children: ReactNode }) {
  const { session, sessionError, sessionLoading } = useApp()
  const [location, setLocation] = useLocation()
  const search = useSearch()

  useEffect(() => {
    if (sessionLoading || session) return

    const next = toSafeNext(`${location}${search}`)
    setLocation(`/login?next=${encodeURIComponent(next)}`, { replace: true })
  }, [location, search, session, sessionLoading, setLocation])

  if (sessionLoading) {
    return (
      <div className='flex min-h-svh items-center justify-center p-6 text-sm text-muted-foreground'>
        Loading session…
      </div>
    )
  }

  if (sessionError) {
    return (
      <div className='flex min-h-svh items-center justify-center p-6 text-sm text-muted-foreground'>
        <Seo title='Auth error' path='/login' noindex />
        Failed to load session: {sessionError}
      </div>
    )
  }

  if (!session) return null

  return <>{children}</>
}
