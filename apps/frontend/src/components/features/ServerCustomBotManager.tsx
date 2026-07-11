import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { serverCustomBotsApi } from '@/data/server-custom-bots'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Bot, KeyRound, Trash2 } from 'lucide-react'
import { useState } from 'react'

export function ServerCustomBotManager({ guildId }: { guildId: string }) {
  const queryClient = useQueryClient()
  const queryKey = ['server-custom-bot', guildId] as const
  const [token, setToken] = useState('')
  const bot = useQuery({ queryKey, queryFn: () => serverCustomBotsApi.get(guildId), retry: false })
  const save = useMutation({
    mutationFn: (value: string) => serverCustomBotsApi.upsert(guildId, value),
    onSuccess: async () => {
      setToken('')
      await queryClient.invalidateQueries({ queryKey })
    }
  })
  const remove = useMutation({
    mutationFn: () => serverCustomBotsApi.delete(guildId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey })
  })

  const submit = (event: React.FormEvent) => {
    event.preventDefault()
    if (token.trim()) save.mutate(token.trim())
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className='flex items-center gap-2'>
          <Bot className='size-4' /> Server bot identity
        </CardTitle>
        <CardDescription>
          Run this server&apos;s existing Flora deployment through a Discord application owned by
          your server. Its profile and branding stay under your control in Discord Developer Portal.
        </CardDescription>
      </CardHeader>
      <CardContent className='space-y-5'>
        <div className='rounded-xl border bg-muted/30 p-4 text-sm text-muted-foreground'>
          Create and customize a bot in Discord Developer Portal, enable the Message Content intent,
          and invite it to this server with the <span className='text-foreground'>bot</span> and{' '}
          <span className='text-foreground'>applications.commands</span> scopes before saving its
          token. Flora binds the token to this server and ignores DMs or other guilds.
        </div>

        {bot.data && (
          <div className='flex flex-wrap items-center justify-between gap-3 rounded-xl border p-4'>
            <div>
              <div className='font-medium'>@{bot.data.bot_username}</div>
              <div className='text-sm text-muted-foreground'>
                {bot.data.running ? 'Handling this server' : 'Configured but stopped'}
              </div>
            </div>
            <Button
              variant='destructive'
              size='sm'
              disabled={remove.isPending}
              onClick={() => {
                if (window.confirm('Remove this server bot and restore central @Flora?'))
                  remove.mutate()
              }}
            >
              <Trash2 /> Remove
            </Button>
          </div>
        )}

        <form className='flex flex-col gap-3 sm:flex-row' onSubmit={submit}>
          <Input
            value={token}
            onChange={(event) => setToken(event.target.value)}
            placeholder={bot.data ? 'New Discord bot token' : 'Discord bot token'}
            type='password'
            autoComplete='off'
          />
          <Button type='submit' disabled={!token.trim() || save.isPending}>
            <KeyRound />{' '}
            {save.isPending ? 'Validating…' : bot.data ? 'Replace token' : 'Use server bot'}
          </Button>
        </form>

        {(save.error || remove.error) && (
          <p className='text-sm text-destructive'>{(save.error || remove.error)?.message}</p>
        )}
        {bot.error && (
          <p className='text-sm text-muted-foreground'>
            {bot.error.message.includes('not enabled')
              ? 'Server custom bots have not been enabled for this guild.'
              : bot.error.message}
          </p>
        )}
      </CardContent>
    </Card>
  )
}
