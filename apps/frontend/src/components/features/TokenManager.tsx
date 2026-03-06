import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import { useApp } from '@/contexts/AppContext'
import { api } from '@/lib/openapi-client'
import { formatDistanceToNow } from 'date-fns'
import { Copy, Plus, Shield, Trash2 } from 'lucide-react'
import { useState } from 'react'

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

export function TokenManager() {
  'use no memo'
  const { tokens, refreshTokens } = useApp()
  const [newToken, setNewToken] = useState<string | null>(null)
  const [tokenLabel, setTokenLabel] = useState<string>('')
  const [error, setError] = useState<string | null>(null)

  const handleCreateToken = async () => {
    setError(null)
    try {
      const res = await api.POST('/tokens/', { body: { label: tokenLabel || undefined } })
      setNewToken(res.data?.token || null)
      setTokenLabel('')
      await refreshTokens()
    } catch (err: any) {
      setNewToken(null)
      setError(err.message || 'Failed to create token')
    }
  }

  const handleDeleteToken = async (tokenId: string) => {
    try {
      await api.DELETE('/tokens/{token_id}', { params: { path: { token_id: tokenId } } })
      await refreshTokens()
    } catch (err: any) {
      setError(err.message || 'Failed to delete token')
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>API Tokens</CardTitle>
        <CardDescription>Manage access tokens for the CLI and external tools.</CardDescription>
      </CardHeader>
      <CardContent className='space-y-6'>
        <div className='flex flex-col gap-4 sm:flex-row'>
          <div className='flex-1'>
            <Input
              placeholder='Token Label (e.g. CI/CD Pipeline)'
              value={tokenLabel}
              onChange={(e) => setTokenLabel(e.target.value)}
            />
          </div>
          <Button onClick={handleCreateToken}>
            <Plus className='mr-2 h-4 w-4' />
            Generate Token
          </Button>
        </div>

        {error && <div className='text-sm text-destructive'>{error}</div>}

        {newToken && (
          <div className='rounded-md border border-green-200 bg-green-50 p-4 dark:bg-green-900/20 dark:border-green-900'>
            <div className='flex items-center justify-between mb-2'>
              <h4 className='text-sm font-semibold text-green-800 dark:text-green-300'>
                Token Generated
              </h4>
              <Button
                variant='ghost'
                size='sm'
                className='h-6 text-green-700 hover:text-green-800 hover:bg-green-100 dark:text-green-400 dark:hover:bg-green-900/40'
                onClick={() => navigator.clipboard.writeText(newToken)}
              >
                <Copy className='h-3 w-3 mr-1' /> Copy
              </Button>
            </div>
            <code className='block w-full overflow-x-auto rounded bg-white p-2 font-mono text-xs text-green-900 border border-green-100 dark:bg-black/40 dark:text-green-100 dark:border-green-900/50'>
              {newToken}
            </code>
            <p className='mt-2 text-xs text-green-700 dark:text-green-400'>
              Make sure to copy this token now. You won't be able to see it again.
            </p>
          </div>
        )}

        <div className='space-y-1'>
          <h4 className='text-sm font-medium text-muted-foreground mb-3'>Active Tokens</h4>
          {tokens.loading && <Skeleton className='h-12 w-full' />}
          {!tokens.loading && !tokens.data?.length && (
            <EmptyState
              icon={Shield}
              title='No active tokens'
              description='Generate a token to get started with the CLI.'
            />
          )}
          {tokens.data?.map((token) => (
            <div
              key={token.token_id}
              className='flex items-center justify-between rounded-md border p-3'
            >
              <div>
                <div className='font-medium text-sm'>{token.label || 'Untitled Token'}</div>
                <div className='text-xs text-muted-foreground'>
                  Created {formatTimeAgo(token.created_at)} • Last used{' '}
                  {formatTimeAgo(token.last_used_at)}
                </div>
              </div>
              <Button
                variant='ghost'
                size='sm'
                onClick={() => handleDeleteToken(token.token_id)}
                className='text-destructive hover:text-destructive hover:bg-destructive/10'
              >
                <Trash2 className='h-4 w-4' />
              </Button>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
