import { DocumentationStack } from '@/components/docs-stack/documentation-stack'
import { DashboardSidebar } from '@/components/sidebar/app-sidebar'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { useApp } from '@/contexts/AppContext'
import { useRecentGuilds } from '@/hooks/use-recent-guilds'
import { Seo } from '@/lib/seo'
import { Clock4 } from 'lucide-react'
import { useMemo } from 'react'
import { match } from 'ts-pattern'
import { useLocation } from 'wouter'

function getGuildInitials(name: string) {
  return name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('')
}

export function Dashboard() {
  const { guilds, session, setSelectedGuild, setView } = useApp()
  const { recentGuildIds } = useRecentGuilds()
  const [, setLocation] = useLocation()
  const recentGuildLimit = 4

  const recentGuilds = useMemo(() => {
    const all = guilds.data ?? []
    const fromRecent = recentGuildIds
      .map((id) => all.find((guild) => guild.id === id))
      .filter((guild): guild is NonNullable<typeof guild> => Boolean(guild))
    if (fromRecent.length >= recentGuildLimit) return fromRecent.slice(0, recentGuildLimit)
    const rest = all.filter((guild) => !fromRecent.some((recent) => recent.id === guild.id))
    return [...fromRecent, ...rest].slice(0, recentGuildLimit)
  }, [guilds.data, recentGuildIds, recentGuildLimit])

  const welcomeUsername = session?.username
  const recentGuildsContent = match(recentGuilds.length)
    .with(0, () => (
      <div className='rounded-xl border border-dashed p-8 text-sm text-muted-foreground'>
        Pick a server from the sidebar to create your recent list.
      </div>
    ))
    .otherwise(() => (
      <div className='grid gap-4 sm:grid-cols-2 lg:grid-cols-4'>
        {recentGuilds.map((guild) => (
          <button
            key={guild.id}
            type='button'
            className='group flex h-full cursor-pointer flex-col rounded-2xl border bg-card p-4 text-left transition hover:-translate-y-0.5 hover:shadow-md'
            onClick={() => {
              setSelectedGuild(guild.id)
              setView('overview')
              setLocation(`/${guild.id}`)
            }}
          >
            <div className='mb-4 flex items-center gap-3'>
              <Avatar className='h-10 w-10'>
                <AvatarImage
                  src={
                    guild.icon
                      ? `https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=128`
                      : undefined
                  }
                />
                <AvatarFallback>{getGuildInitials(guild.name)}</AvatarFallback>
              </Avatar>
              <div className='font-medium'>{guild.name}</div>
            </div>
            <div className='inline-flex items-center gap-2 text-sm text-muted-foreground group-hover:text-foreground'>
              <Clock4 className='h-4 w-4' />
              Continue managing guild
            </div>
          </button>
        ))}
      </div>
    ))

  return (
    <>
      <Seo
        title='Guild dashboard'
        description='Manage flora guild deployments, logs, and settings from one dashboard.'
        path='/'
      />
      <SidebarProvider>
        <div className='relative flex h-dvh w-full'>
          <DashboardSidebar />
          <SidebarInset className='flex min-w-0 flex-1 flex-col'>
            <div className='absolute top-3 left-3 z-40 lg:hidden'>
              <SidebarTrigger />
            </div>
            <div className='flex-1 overflow-y-auto p-4 md:p-6 lg:p-8'>
              <div className='mx-auto flex w-full max-w-6xl flex-col gap-12 pb-8'>
                <section className='space-y-4'>
                  <div className='space-y-1'>
                    {welcomeUsername && (
                      <p className='text-sm font-medium text-muted-foreground'>
                        Welcome back, @{welcomeUsername}
                      </p>
                    )}
                    <h2 className='text-2xl font-semibold tracking-tight'>Jump back in</h2>
                    <p className='text-sm text-muted-foreground'>
                      Your most recently used servers.
                    </p>
                  </div>
                  {recentGuildsContent}
                </section>

                <section className='space-y-6'>
                  <div className='space-y-1'>
                    <h2 className='text-2xl font-semibold tracking-tight'>Documentation</h2>
                    <p className='text-sm text-muted-foreground'>
                      Need to invite the bot first?{' '}
                      <a
                        className='font-medium text-foreground underline-offset-4 transition hover:underline'
                        href='https://discord.com/oauth2/authorize?client_id=1446796323113140264&permissions=0&integration_type=0&scope=applications.commands+bot'
                        rel='noreferrer'
                        target='_blank'
                      >
                        Invite flora
                      </a>
                      .
                    </p>
                  </div>

                  <DocumentationStack />
                </section>
              </div>
            </div>
          </SidebarInset>
        </div>
      </SidebarProvider>
    </>
  )
}
