import { Button } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { customBotsApi } from '@/data/custom-bots'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Bot, Code2, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useLocation } from 'wouter'

const queryKey = ['custom-bots'] as const

export function CustomBotManager() {
  const queryClient = useQueryClient()
  const [, setLocation] = useLocation()
  const [label, setLabel] = useState('')
  const [token, setToken] = useState('')
  const bots = useQuery({ queryKey, queryFn: customBotsApi.list, retry: false })
  const createBot = useMutation({
    mutationFn: customBotsApi.create,
    onSuccess: async () => {
      setLabel('')
      setToken('')
      await queryClient.invalidateQueries({ queryKey })
    }
  })
  const deleteBot = useMutation({
    mutationFn: customBotsApi.delete,
    onSuccess: () => queryClient.invalidateQueries({ queryKey })
  })
  const hasBot = (bots.data?.length ?? 0) > 0

  const submit = (event: React.FormEvent) => {
    event.preventDefault()
    if (!token.trim()) return
    createBot.mutate({ label: label.trim() || undefined, token: token.trim() })
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className='flex items-center gap-2'>
          <Bot className='size-4' /> User Bots
        </CardTitle>
        <CardDescription>
          Run a Discord bot owned by your account with DMs, global commands, and an isolated Flora
          deployment. One User Bot per account.
        </CardDescription>
      </CardHeader>
      <CardContent className='space-y-5'>
        {hasBot ? (
          <p className='text-sm text-muted-foreground'>
            Your account already has a User Bot. Delete it to create a new one.
          </p>
        ) : (
          <form
            className='grid gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,2fr)_auto]'
            onSubmit={submit}
          >
            <Input
              value={label}
              onChange={(event) => setLabel(event.target.value)}
              placeholder='Label (optional)'
              autoComplete='off'
            />
            <Input
              value={token}
              onChange={(event) => setToken(event.target.value)}
              placeholder='Discord bot token'
              type='password'
              autoComplete='off'
            />
            <Button type='submit' disabled={!token.trim() || createBot.isPending}>
              <Plus /> {createBot.isPending ? 'Adding…' : 'Add bot'}
            </Button>
          </form>
        )}

        {createBot.error && <p className='text-sm text-destructive'>{createBot.error.message}</p>}
        {bots.error && (
          <p className='text-sm text-muted-foreground'>
            {bots.error.message.includes('not enabled')
              ? 'User Bots have not been enabled for your account.'
              : bots.error.message}
          </p>
        )}
        <div className='grid gap-3'>
          {bots.data?.map((bot) => (
            <Card key={bot.id} size='sm' className='rounded-xl'>
              <CardHeader>
                <CardTitle>{bot.label || bot.bot_username}</CardTitle>
                <CardDescription>
                  @{bot.bot_username} ·{' '}
                  {bot.running ? 'Running' : bot.has_deployment ? 'Stopped' : 'Not deployed'}
                </CardDescription>
                <CardAction className='flex gap-2'>
                  <Button
                    variant='outline'
                    size='sm'
                    onClick={() => setLocation(`/bots/${bot.id}/editor`)}
                  >
                    <Code2 /> Editor
                  </Button>
                  <Button
                    variant='destructive'
                    size='icon-sm'
                    aria-label={`Delete ${bot.label || bot.bot_username}`}
                    onClick={() => {
                      if (window.confirm(`Delete ${bot.label || bot.bot_username}?`)) {
                        deleteBot.mutate(bot.id)
                      }
                    }}
                  >
                    <Trash2 />
                  </Button>
                </CardAction>
              </CardHeader>
            </Card>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
