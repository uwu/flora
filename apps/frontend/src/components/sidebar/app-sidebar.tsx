'use client'

import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarTrigger,
  useSidebar
} from '@/components/ui/sidebar'
import { Skeleton } from '@/components/ui/skeleton'
import { useApp } from '@/contexts/AppContext'
import { useRecentGuilds } from '@/hooks/use-recent-guilds'
import { cn } from '@/lib/utils'
import { BookText, Database, FileCode2, ListChecks, Shield } from 'lucide-react'
import { domAnimation, LazyMotion, m, useReducedMotion } from 'motion/react'
import { match } from 'ts-pattern'
import { useLocation } from 'wouter'
import DashboardNavigation, { type Route } from './nav-main'
import { NavUser } from './nav-user'

function getGuildInitials(name: string) {
  return name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('')
}

export function DashboardSidebar() {
  const { state, isMobile, setOpenMobile } = useSidebar()
  const isCollapsed = state === 'collapsed'
  const { session, guilds, selectedGuild, setSelectedGuild, setSession, view, setView } = useApp()
  const { pushRecentGuild } = useRecentGuilds()
  const [, setLocation] = useLocation()
  const prefersReducedMotion = useReducedMotion()

  const handleGuildClick = (guildId: string) => {
    setSelectedGuild(guildId)
    setView('guild')
    pushRecentGuild(guildId)
    if (isMobile) setOpenMobile(false)
  }

  const handleLogoutClick = () => {
    setSession(null)
    setLocation('/login')
    if (isMobile) setOpenMobile(false)
  }

  const routes: Route[] = guilds.data?.map((guild) => ({
    id: guild.id,
    title: guild.name,
    icon: (
      <Avatar className='h-6 w-6 text-[10px]'>
        <AvatarImage
          src={guild.icon
            ? `https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=128`
            : undefined}
        />
        <AvatarFallback>{getGuildInitials(guild.name)}</AvatarFallback>
      </Avatar>
    ),
    isActive: selectedGuild === guild.id,
    onClick: () => handleGuildClick(guild.id),
    subs: [
      {
        title: 'Overview',
        onClick: () => setLocation(`/${guild.id}`),
        isActive: view === 'overview' && selectedGuild === guild.id,
        icon: <BookText className='h-4 w-4' />
      },
      {
        title: 'Editor',
        onClick: () => setLocation(`/${guild.id}/editor`),
        isActive: view === 'editor' && selectedGuild === guild.id,
        icon: <FileCode2 className='h-4 w-4' />
      },
      {
        title: 'Deployments',
        onClick: () => setLocation(`/${guild.id}/deployments`),
        isActive: view === 'deployments' && selectedGuild === guild.id,
        icon: <ListChecks className='h-4 w-4' />
      },
      {
        title: 'KV',
        onClick: () => setLocation(`/${guild.id}/kv`),
        isActive: view === 'kv' && selectedGuild === guild.id,
        icon: <Database className='h-4 w-4' />
      }
    ]
  })) || []

  const guildsContent = match({ isLoading: guilds.loading, hasRoutes: routes.length > 0 })
    .with({ isLoading: true }, () => (
      <div className='space-y-2 px-2'>
        <Skeleton className='h-8 w-full' />
        <Skeleton className='h-8 w-full' />
        <Skeleton className='h-8 w-full' />
      </div>
    ))
    .with({ isLoading: false, hasRoutes: false }, () => (
      <div
        className={cn(
          'flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed p-4 text-center',
          !isCollapsed && 'mt-3'
        )}
      >
        <Shield className='h-5 w-5 text-muted-foreground' />
        {!isCollapsed && <p className='text-xs text-muted-foreground'>No guilds found</p>}
      </div>
    ))
    .otherwise(() => <DashboardNavigation routes={routes} />)

  return (
    <Sidebar variant='inset' collapsible='icon'>
      <SidebarHeader
        className={cn(
          'flex md:pt-3.5',
          isCollapsed
            ? 'flex-row items-center justify-between gap-y-4 md:flex-col md:items-start md:justify-start'
            : 'flex-row items-center justify-between'
        )}
      >
        <button
          type='button'
          className='flex cursor-pointer items-center gap-2'
          onClick={() => {
            setView('guild')
            setLocation('/')
          }}
        >
          <img src='/logo.svg' alt='flora logo' className='h-8 w-8' />
          {!isCollapsed && (
            <span className='font-semibold text-black dark:text-white'>
              flora
            </span>
          )}
        </button>

        <LazyMotion features={domAnimation}>
          <m.div
            key={isCollapsed ? 'header-collapsed' : 'header-expanded'}
            className={cn(
              'flex items-center gap-2',
              isCollapsed ? 'flex-row md:flex-col-reverse' : 'flex-row'
            )}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: prefersReducedMotion ? 0 : 0.8 }}
          >
            <SidebarTrigger />
          </m.div>
        </LazyMotion>
      </SidebarHeader>
      <SidebarContent className={cn('gap-4 px-2', !isCollapsed && 'py-4')}>
        <div>{guildsContent}</div>
      </SidebarContent>
      <SidebarFooter className='px-2'>
        {session && (
          <NavUser
            user={session}
            onSettingsClick={() => {
              setView('guild')
              setLocation('/settings')
              if (isMobile) setOpenMobile(false)
            }}
            onLogoutClick={handleLogoutClick}
          />
        )}
      </SidebarFooter>
    </Sidebar>
  )
}
