'use client'

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuItem as SidebarMenuSubItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  useSidebar
} from '@/components/ui/sidebar'
import { cn } from '@/lib/utils'
import { ChevronDown, ChevronUp } from 'lucide-react'
import type React from 'react'
import { useState } from 'react'

export type Route = {
  id: string
  title: string
  icon?: React.ReactNode
  isActive?: boolean
  onClick?: () => void
  href?: string
  subs?: {
    title: string
    onClick?: () => void
    isActive?: boolean
    icon?: React.ReactNode
  }[]
}

export default function DashboardNavigation({ routes }: { routes: Route[] }) {
  const { state } = useSidebar()
  const isCollapsed = state === 'collapsed'
  const [openCollapsible, setOpenCollapsible] = useState<string | null>(null)
  const activeRouteId =
    routes.find((route) => route.subs?.some((subRoute) => subRoute.isActive))?.id ?? null

  return (
    <SidebarMenu>
      {routes.map((route) => {
        const effectiveOpenRoute = openCollapsible ?? activeRouteId
        const isOpen = !isCollapsed && effectiveOpenRoute === route.id
        const hasSubRoutes = !!route.subs?.length

        return (
          <SidebarMenuItem key={route.id}>
            {hasSubRoutes
              ? (
                <Collapsible
                  open={isOpen}
                  onOpenChange={(open) => setOpenCollapsible(open ? route.id : null)}
                  className='w-full'
                >
                  <CollapsibleTrigger
                    render={
                      <SidebarMenuButton
                        className={cn(
                          'w-full justify-between group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:!p-0',
                          isOpen && 'bg-sidebar-accent text-sidebar-accent-foreground'
                        )}
                        tooltip={route.title}
                      >
                        <div className='flex items-center gap-2'>
                          {route.icon}
                          {!isCollapsed && <span>{route.title}</span>}
                        </div>
                        {!isCollapsed &&
                          (isOpen
                            ? <ChevronUp className='size-4' />
                            : <ChevronDown className='size-4' />)}
                      </SidebarMenuButton>
                    }
                  />

                  <CollapsibleContent>
                    <SidebarMenuSub>
                      {route.subs?.map((subRoute) => (
                        <SidebarMenuSubItem key={`${route.id}-${subRoute.title}`}>
                          <SidebarMenuSubButton
                            isActive={subRoute.isActive}
                            onClick={() => {
                              setOpenCollapsible(route.id)
                              subRoute.onClick?.()
                            }}
                            render={
                              <button className='flex w-full cursor-pointer items-center gap-2'>
                                {subRoute.icon}
                                <span>{subRoute.title}</span>
                              </button>
                            }
                          />
                        </SidebarMenuSubItem>
                      ))}
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </Collapsible>
              )
              : (
                <SidebarMenuButton
                  tooltip={route.title}
                  isActive={route.isActive}
                  onClick={route.onClick}
                  className='cursor-pointer group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:!p-0'
                >
                  {route.icon}
                  {!isCollapsed && <span>{route.title}</span>}
                </SidebarMenuButton>
              )}
          </SidebarMenuItem>
        )
      })}
    </SidebarMenu>
  )
}
