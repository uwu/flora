'use client'

import { Avatar } from '@/components/ui/avatar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar
} from '@/components/ui/sidebar'
import type { components } from '@/lib/openapi-schema'
import { redirectToLogin } from '@/lib/utils'
import { ChevronsUpDown, LogOut, Settings } from 'lucide-react'

type AuthUser = components['schemas']['AuthUser']

interface NavUserProps {
  user: AuthUser
  onSettingsClick: () => void
  onLogoutClick: () => void
}

export function NavUser({ user, onSettingsClick, onLogoutClick }: NavUserProps) {
  const { isMobile, state } = useSidebar()
  const isCollapsed = state === 'collapsed'

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <SidebarMenuButton
                size='lg'
                className='data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground'
              >
                <Avatar
                  name={user.global_name || user.username}
                  userId={user.id}
                  avatarHash={user.avatar}
                  className='h-8 w-8 rounded-lg'
                />
                {!isCollapsed && (
                  <>
                    <div className='grid flex-1 text-left text-sm leading-tight'>
                      <span className='truncate font-semibold'>
                        {user.global_name || user.username}
                      </span>
                      <span className='truncate text-xs text-muted-foreground'>Manage Account</span>
                    </div>
                    <ChevronsUpDown className='ml-auto size-4' />
                  </>
                )}
              </SidebarMenuButton>
            }
          />
          <DropdownMenuContent
            className='w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg mb-4'
            align='start'
            side={isMobile ? 'bottom' : 'right'}
            sideOffset={4}
          >
            <DropdownMenuGroup>
              <DropdownMenuLabel className='p-0 font-normal'>
                <div className='flex items-center gap-2 px-1 py-1.5 text-left text-sm'>
                  <Avatar
                    name={user.global_name || user.username}
                    userId={user.id}
                    avatarHash={user.avatar}
                    className='h-8 w-8 rounded-lg'
                  />
                  <div className='grid flex-1 text-left text-sm leading-tight'>
                    <span className='truncate font-semibold'>
                      {user.global_name || user.username}
                    </span>
                    <span className='truncate text-xs text-muted-foreground'>{user.username}</span>
                  </div>
                </div>
              </DropdownMenuLabel>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={onSettingsClick} className='cursor-pointer'>
              <Settings className='mr-2 h-4 w-4' />
              Settings
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={onLogoutClick}
              className='cursor-pointer text-destructive focus:text-destructive'
            >
              <LogOut className='mr-2 h-4 w-4' />
              Log out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
