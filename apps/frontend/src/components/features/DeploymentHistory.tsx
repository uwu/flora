import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useApp } from '@/contexts/AppContext'
import type { components } from '@/lib/openapi-schema'
import { formatDistanceToNow } from 'date-fns'
import { Code2, History } from 'lucide-react'

function formatTimeAgo(value?: string | null) {
  if (!value) return 'never'
  return formatDistanceToNow(new Date(value), { addSuffix: true })
}

function EmptyState({
  icon: Icon,
  title,
  description
}: {
  icon: React.ComponentType<{ className?: string }>
  title: string
  description: string
}) {
  return (
    <div className='flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-8 text-center animate-in fade-in zoom-in-95'>
      <div className='rounded-full bg-muted p-3'>
        <Icon className='h-6 w-6 text-muted-foreground' />
      </div>
      <div className='font-medium mt-2'>{title}</div>
      <div className='text-muted-foreground text-sm max-w-xs'>{description}</div>
    </div>
  )
}

type Deployment = components['schemas']['DeploymentResponse']

export function DeploymentHistory({ deploymentsOverride }: { deploymentsOverride?: Deployment[] }) {
  const { deployments, selectedGuild, guilds } = useApp()
  const data = deploymentsOverride ?? deployments.data ?? []

  return (
    <Card>
      <CardHeader>
        <CardTitle>Deployment History</CardTitle>
        <CardDescription>Recent updates to your guild bots.</CardDescription>
      </CardHeader>
      <CardContent>
        {!data.length
          ? (
            <EmptyState
              icon={History}
              title='No deployments'
              description="You haven't deployed any code yet."
            />
          )
          : (
            <div className='space-y-4'>
              {data
                .filter((d) => d.guild_id === selectedGuild || !selectedGuild)
                .map((dep) => (
                  <div
                    key={dep.guild_id}
                    className='flex items-center justify-between rounded-lg border p-4 transition-colors hover:bg-muted/50'
                  >
                    <div className='flex items-center gap-4'>
                      <div className='rounded-full bg-primary/10 p-2 text-primary'>
                        <Code2 className='h-4 w-4' />
                      </div>
                      <div>
                        <p className='font-medium text-sm'>
                          {guilds.data?.find((g) => g.id === dep.guild_id)?.name || dep.guild_id}
                        </p>
                        <p className='text-xs text-muted-foreground'>
                          Deployed {formatTimeAgo(dep.updated_at)}
                        </p>
                      </div>
                    </div>
                    <div className='flex items-center gap-3'>
                      <Badge variant='secondary' className='font-mono text-xs'>
                        auto-detected
                      </Badge>
                    </div>
                  </div>
                ))}
            </div>
          )}
      </CardContent>
    </Card>
  )
}
