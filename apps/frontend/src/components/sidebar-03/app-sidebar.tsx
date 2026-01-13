'use client'

import { Avatar } from '@/components/ui/avatar'
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
import { cn } from '@/lib/utils'
import { redirectToLogin } from '@/lib/utils'
import { motion } from 'framer-motion'
import { BookText, FileCode2, ListChecks, Shield } from 'lucide-react'
import { useLocation } from 'wouter'
import { Separator } from '../ui/separator'
import DashboardNavigation, { type Route } from './nav-main'
import { NavUser } from './nav-user'

export function DashboardSidebar() {
  const { state, isMobile, setOpenMobile } = useSidebar()
  const isCollapsed = state === 'collapsed'
  const { session, guilds, selectedGuild, setSelectedGuild, view, setView } = useApp()
  const [, setLocation] = useLocation()

  const handleGuildClick = (guildId: string) => {
    setSelectedGuild(guildId)
    setView('guild')
    if (isMobile) setOpenMobile(false)
  }

  const routes: Route[] = guilds.data?.map((guild) => ({
    id: guild.id,
    title: guild.name,
    icon: (
      <Avatar
        name={guild.name}
        guildId={guild.id}
        iconHash={guild.icon}
        className='h-6 w-6 text-[10px]'
      />
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
      }
    ]
  })) || []

  return (
    <Sidebar variant='floating' collapsible='icon'>
      <SidebarHeader
        className={cn(
          'flex md:pt-3.5',
          isCollapsed
            ? 'flex-row items-center justify-between gap-y-4 md:flex-col md:items-start md:justify-start'
            : 'flex-row items-center justify-between'
        )}
      >
        <div
          className='flex cursor-pointer items-center gap-2'
          onClick={() => {
            setView('guild')
            setLocation('/')
          }}
        >
          <img src='/logo.png' alt='logo' className='h-8 w-8 rounded-lg object-cover' />
          {!isCollapsed && <span className='font-semibold text-black dark:text-white'>flora</span>}
        </div>

        <motion.div
          key={isCollapsed ? 'header-collapsed' : 'header-expanded'}
          className={cn(
            'flex items-center gap-2',
            isCollapsed ? 'flex-row md:flex-col-reverse' : 'flex-row'
          )}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.8 }}
        >
          <SidebarTrigger />
        </motion.div>
      </SidebarHeader>
      <SidebarContent className={cn('gap-4 px-2', !isCollapsed && 'py-4')}>
        <div>
          {!isCollapsed
            ? (
              <div className='px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground'>
                Your Guilds
              </div>
            )
            : <Separator />}
          {guilds.loading
            ? (
              <div className='space-y-2 px-2'>
                <Skeleton className='h-8 w-full' />
                <Skeleton className='h-8 w-full' />
                <Skeleton className='h-8 w-full' />
              </div>
            )
            : routes.length === 0
            ? (
              <div className='flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed p-4 text-center'>
                <Shield className='h-5 w-5 text-muted-foreground' />
                {!isCollapsed && <p className='text-xs text-muted-foreground'>No guilds found</p>}
              </div>
            )
            : <DashboardNavigation routes={routes} />}
        </div>
      </SidebarContent>
      <SidebarFooter className='px-2'>
        {session && (
          <NavUser
            user={session}
            onSettingsClick={() => setView('user-settings')}
            onLogoutClick={redirectToLogin}
          />
        )}
      </SidebarFooter>
    </Sidebar>
  )
}
