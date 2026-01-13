import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useApp } from '@/contexts/AppContext'
import { formatDistanceToNow } from 'date-fns'
import { Activity, Box, Clock } from 'lucide-react'

function formatTimeAgo(value?: string | null) {
  if (!value) return 'never'
  return formatDistanceToNow(new Date(value), { addSuffix: true })
}

function MetricCard({
  icon: Icon,
  label,
  value,
  description
}: {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value: string
  description: string
}) {
  return (
    <Card className='overflow-hidden'>
      <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2'>
        <CardTitle className='text-sm font-medium text-muted-foreground'>{label}</CardTitle>
        <Icon className='h-4 w-4 text-muted-foreground' />
      </CardHeader>
      <CardContent>
        <div className='text-2xl font-bold'>{value}</div>
        <p className='text-xs text-muted-foreground mt-1'>{description}</p>
      </CardContent>
    </Card>
  )
}

export function MetricCards() {
  const { deployments, selectedGuild } = useApp()

  const currentDeployment = deployments.data?.find((d) => d.guild_id === selectedGuild)
  const language = 'typescript'

  return (
    <div className='grid gap-4 md:grid-cols-3'>
      <MetricCard
        icon={Activity}
        label='Total Deployments'
        value={deployments.data?.length.toString() || '0'}
        description='Across all guilds'
      />
      <MetricCard
        icon={Box}
        label='Current Language'
        value={language === 'typescript' ? 'TS' : 'JS'}
        description={language}
      />
      <MetricCard
        icon={Clock}
        label='Last Updated'
        value={currentDeployment?.updated_at
          ? formatTimeAgo(currentDeployment.updated_at)
          : 'Never'}
        description='Deployment age'
      />
    </div>
  )
}
