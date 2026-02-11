import { DashboardSidebar } from '@/components/sidebar-03/app-sidebar'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { useEffect, useState } from 'react'
import { useParams } from 'wouter'

type GuildBotBinding = {
  guild_id: string
  owner_user_id: string
  bot_user_id: string
  bot_username: string
  application_id: string
  created_at: string
  updated_at: string
}

export function Settings() {
  const { guildId } = useParams<{ guildId: string }>()
  const { setView, setSelectedGuild } = useApp()
  const [binding, setBinding] = useState<GuildBotBinding | null>(null)
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [token, setToken] = useState('')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (guildId) setSelectedGuild(guildId)
    setView('overview')
  }, [guildId, setSelectedGuild, setView])

  const loadBinding = async () => {
    if (!guildId) return
    setLoading(true)
    setError(null)
    try {
      const res = await fetch(`/api/guild-bots/${guildId}`, {
        credentials: 'include'
      })

      if (res.status === 404) {
        setBinding(null)
        return
      }
      if (!res.ok) {
        const body = await res.text()
        throw new Error(body || 'Failed to load BYOB binding')
      }

      setBinding(await res.json())
    } catch (err: any) {
      setError(err.message || 'Failed to load BYOB binding')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadBinding()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [guildId])

  const connectBot = async () => {
    if (!guildId || !token.trim()) return
    setSaving(true)
    setError(null)
    try {
      const res = await fetch(`/api/guild-bots/${guildId}`, {
        method: 'PUT',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ bot_token: token.trim() })
      })
      if (!res.ok) {
        const body = await res.text()
        throw new Error(body || 'Failed to save BYOB binding')
      }
      setBinding(await res.json())
      setToken('')
    } catch (err: any) {
      setError(err.message || 'Failed to save BYOB binding')
    } finally {
      setSaving(false)
    }
  }

  const disconnectBot = async () => {
    if (!guildId) return
    setSaving(true)
    setError(null)
    try {
      const res = await fetch(`/api/guild-bots/${guildId}`, {
        method: 'DELETE',
        credentials: 'include'
      })
      if (!res.ok && res.status !== 404) {
        const body = await res.text()
        throw new Error(body || 'Failed to remove BYOB binding')
      }
      setBinding(null)
    } catch (err: any) {
      setError(err.message || 'Failed to remove BYOB binding')
    } finally {
      setSaving(false)
    }
  }

  return (
    <SidebarProvider>
      <div className='flex h-screen w-full bg-background text-foreground overflow-hidden font-sans'>
        <DashboardSidebar />
        <SidebarInset className='flex min-w-0 flex-1 flex-col bg-background'>
          <header className='sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/95 px-6 backdrop-blur supports-[backdrop-filter]:bg-background/60'>
            <SidebarTrigger className='lg:hidden -ml-2' />
            <div className='font-medium'>Settings</div>
            <div className='ml-auto' />
          </header>
          <div className='flex-1 p-4 md:p-6 lg:p-8 overflow-y-auto'>
            <div className='mx-auto max-w-3xl space-y-4'>
              <Card>
                <CardHeader>
                  <CardTitle>Bring Your Own Bot</CardTitle>
                  <CardDescription>
                    Bind a custom Discord bot token for this guild only.
                  </CardDescription>
                </CardHeader>
                <CardContent className='space-y-4'>
                  <div className='text-sm text-muted-foreground'>
                    Guild: <span className='font-mono'>{guildId ?? 'unknown'}</span>
                  </div>

                  {loading && <div className='text-sm text-muted-foreground'>Loading binding…</div>}

                  {!loading && binding && (
                    <div className='rounded-md border p-3 text-sm space-y-1'>
                      <div>
                        Bot: <span className='font-medium'>{binding.bot_username}</span> (
                        <span className='font-mono'>{binding.bot_user_id}</span>)
                      </div>
                      <div>
                        App ID: <span className='font-mono'>{binding.application_id}</span>
                      </div>
                    </div>
                  )}

                  {!loading && !binding && (
                    <div className='text-sm text-muted-foreground'>
                      No BYOB bot bound for this guild.
                    </div>
                  )}

                  <Input
                    placeholder='Bot <token>'
                    value={token}
                    onChange={(e) => setToken(e.target.value)}
                  />

                  {error && <div className='text-sm text-destructive'>{error}</div>}

                  <div className='flex gap-2'>
                    <Button onClick={connectBot} disabled={saving || !token.trim()}>
                      {saving ? 'Saving…' : 'Bind Bot'}
                    </Button>
                    <Button
                      variant='outline'
                      onClick={disconnectBot}
                      disabled={saving || !binding}
                    >
                      Unbind Bot
                    </Button>
                    <Button variant='ghost' onClick={loadBinding} disabled={saving || loading}>
                      Refresh
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        </SidebarInset>
      </div>
    </SidebarProvider>
  )
}
