import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from '@/components/ui/table'
import { $api } from '@/lib/openapi-client'
import type { components } from '@/lib/openapi-schema'
import { cn } from '@/lib/utils'
import { Database, KeyRound, Plus, RefreshCw, Trash2 } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'

const KEY_LIMIT = 200

type KvStore = components['schemas']['KvStore']
type RawKvKeyInfo = components['schemas']['RawKvKeyInfo']

function formatExpiration(expiration?: number | null) {
  if (!expiration) return 'never'
  return new Date(expiration * 1000).toLocaleString()
}

function formatMetadata(metadata?: unknown) {
  if (metadata == null) return ''
  try {
    return JSON.stringify(metadata, null, 2)
  } catch {
    return String(metadata)
  }
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

export function KvManager({ guildId }: { guildId: string }) {
  'use no memo'
  const [storeName, setStoreName] = useState('')
  const [selectedStore, setSelectedStore] = useState<string>('')
  const [keyPrefix, setKeyPrefix] = useState('')
  const [keyName, setKeyName] = useState('')
  const [keyValue, setKeyValue] = useState('')
  const [keyMetadata, setKeyMetadata] = useState('')
  const [keyExpiration, setKeyExpiration] = useState('')
  const [selectedKey, setSelectedKey] = useState<RawKvKeyInfo | null>(null)
  const [error, setError] = useState<string | null>(null)

  const storesQuery = $api.useQuery(
    'get',
    '/kv/stores',
    {
      params: {
        query: {
          guild_id: guildId
        }
      }
    },
    {
      enabled: !!guildId,
      refetchOnWindowFocus: false
    }
  )

  const keysQuery = $api.useQuery(
    'get',
    '/kv/{guild_id}/{store_name}',
    {
      params: {
        path: { guild_id: guildId, store_name: selectedStore },
        query: {
          prefix: keyPrefix.trim() ? keyPrefix.trim() : undefined,
          limit: KEY_LIMIT
        }
      }
    },
    {
      enabled: !!guildId && !!selectedStore,
      refetchOnWindowFocus: false
    }
  )

  const valueQuery = $api.useQuery(
    'get',
    '/kv/{guild_id}/{store_name}/{key}',
    {
      params: {
        path: {
          guild_id: guildId,
          store_name: selectedStore,
          key: selectedKey?.name ?? ''
        }
      }
    },
    {
      enabled: !!guildId && !!selectedStore && !!selectedKey?.name,
      refetchOnWindowFocus: false
    }
  )

  const createStoreMutation = $api.useMutation('post', '/kv/stores', {
    onSuccess: () => {
      setStoreName('')
      void storesQuery.refetch()
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to create store')
    }
  })

  const deleteStoreMutation = $api.useMutation('delete', '/kv/stores/{guild_id}/{store_name}', {
    onSuccess: () => {
      void storesQuery.refetch()
      setSelectedStore('')
      setSelectedKey(null)
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to delete store')
    }
  })

  const setKeyMutation = $api.useMutation('put', '/kv/{guild_id}/{store_name}/{key}', {
    onSuccess: () => {
      void keysQuery.refetch()
      setSelectedKey(null)
      setKeyName('')
      setKeyValue('')
      setKeyMetadata('')
      setKeyExpiration('')
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to save key')
    }
  })

  const deleteKeyMutation = $api.useMutation('delete', '/kv/{guild_id}/{store_name}/{key}', {
    onSuccess: () => {
      void keysQuery.refetch()
      setSelectedKey(null)
      setKeyName('')
      setKeyValue('')
      setKeyMetadata('')
      setKeyExpiration('')
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to delete key')
    }
  })

  const stores = storesQuery.data ?? []
  const keys = keysQuery.data?.keys ?? []

  useEffect(() => {
    if (!storesQuery.data) return
    if (storesQuery.data.length === 0) {
      setSelectedStore('')
      return
    }
    if (!selectedStore || !storesQuery.data.some((store) => store.store_name === selectedStore)) {
      setSelectedStore(storesQuery.data[0]?.store_name ?? '')
    }
  }, [selectedStore, storesQuery.data])

  useEffect(() => {
    setSelectedKey(null)
    setKeyName('')
    setKeyValue('')
    setKeyMetadata('')
    setKeyExpiration('')
  }, [selectedStore])

  useEffect(() => {
    if (!selectedKey?.name) return
    if (!valueQuery.data) return
    setKeyValue(valueQuery.data.value ?? '')
  }, [selectedKey, valueQuery.data])

  const storeSummary = useMemo(() => {
    if (!selectedStore) return 'Select a store to view keys.'
    if (!keysQuery.data) return 'Loading keys…'
    if (keysQuery.data.listComplete) return `${keys.length} keys loaded`
    return `${keys.length} keys loaded (more available)`
  }, [keys.length, keysQuery.data, selectedStore])

  const handleCreateStore = () => {
    setError(null)
    const name = storeName.trim()
    if (!name) {
      setError('Store name required')
      return
    }
    void createStoreMutation.mutateAsync({ body: { guild_id: guildId, store_name: name } })
  }

  const handleDeleteStore = (store: KvStore) => {
    setError(null)
    if (!confirm(`Delete store ${store.store_name}? This removes all keys.`)) return
    void deleteStoreMutation.mutateAsync({
      params: { path: { guild_id: store.guild_id, store_name: store.store_name } }
    })
  }

  const handleSelectKey = (keyInfo: RawKvKeyInfo) => {
    setSelectedKey(keyInfo)
    setKeyName(keyInfo.name)
    setKeyMetadata(formatMetadata(keyInfo.metadata))
    setKeyExpiration(keyInfo.expiration ? String(keyInfo.expiration) : '')
    setError(null)
  }

  const handleSaveKey = () => {
    setError(null)
    if (!selectedStore) {
      setError('Select a store')
      return
    }

    const name = keyName.trim()
    if (!name) {
      setError('Key name required')
      return
    }

    let metadata: unknown | undefined
    const metadataInput = keyMetadata.trim()
    if (metadataInput) {
      try {
        metadata = JSON.parse(metadataInput)
      } catch {
        setError('Metadata must be valid JSON')
        return
      }
    }

    let expiration: number | null | undefined
    const expirationInput = keyExpiration.trim()
    if (expirationInput) {
      const parsed = Number(expirationInput)
      if (!Number.isFinite(parsed)) {
        setError('Expiration must be a unix timestamp')
        return
      }
      expiration = Math.trunc(parsed)
    }

    void setKeyMutation.mutateAsync({
      params: {
        path: {
          guild_id: guildId,
          store_name: selectedStore,
          key: name
        }
      },
      body: {
        value: keyValue,
        metadata,
        expiration
      }
    })
  }

  const handleDeleteKey = (keyInfo: RawKvKeyInfo) => {
    setError(null)
    if (!selectedStore) return
    if (!confirm(`Delete key ${keyInfo.name}?`)) return
    void deleteKeyMutation.mutateAsync({
      params: {
        path: {
          guild_id: guildId,
          store_name: selectedStore,
          key: keyInfo.name
        }
      }
    })
  }

  const handleClearEditor = () => {
    setSelectedKey(null)
    setKeyName('')
    setKeyValue('')
    setKeyMetadata('')
    setKeyExpiration('')
    setError(null)
  }

  return (
    <div className='space-y-6'>
      {error && <div className='text-sm text-destructive'>{error}</div>}
      <div className='grid gap-6 lg:grid-cols-[320px_1fr]'>
        <Card>
          <CardHeader>
            <CardTitle>Stores</CardTitle>
            <CardDescription>Create and manage KV stores.</CardDescription>
          </CardHeader>
          <CardContent className='space-y-4'>
            <div className='flex flex-col gap-3'>
              <Input
                placeholder='Store name'
                value={storeName}
                onChange={(event) => setStoreName(event.target.value)}
              />
              <Button onClick={handleCreateStore}>
                <Plus className='mr-2 h-4 w-4' />
                Create store
              </Button>
            </div>

            {storesQuery.isLoading && <Skeleton className='h-10 w-full' />}

            {!storesQuery.isLoading && stores.length === 0 && (
              <EmptyState
                icon={Database}
                title='No stores yet'
                description='Create a store to begin saving data.'
              />
            )}

            <div className='space-y-2'>
              {stores.map((store) => (
                <div
                  key={store.id}
                  className={cn(
                    'flex items-center justify-between rounded-md border px-3 py-2 text-sm',
                    selectedStore === store.store_name && 'border-primary/50 bg-primary/5'
                  )}
                >
                  <button
                    type='button'
                    className='flex-1 text-left font-medium'
                    onClick={() => {
                      setSelectedStore(store.store_name)
                      setSelectedKey(null)
                    }}
                  >
                    {store.store_name}
                  </button>
                  <Button
                    variant='ghost'
                    size='icon-sm'
                    onClick={() => handleDeleteStore(store)}
                    className='text-destructive hover:text-destructive'
                  >
                    <Trash2 className='h-4 w-4' />
                  </Button>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <div className='space-y-6'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between'>
              <div>
                <CardTitle>Keys</CardTitle>
                <CardDescription>{storeSummary}</CardDescription>
              </div>
              <Button
                variant='outline'
                size='sm'
                onClick={() => keysQuery.refetch()}
                disabled={!selectedStore || keysQuery.isFetching}
              >
                <RefreshCw className='mr-2 h-4 w-4' />
                Refresh
              </Button>
            </CardHeader>
            <CardContent className='space-y-4'>
              <Input
                placeholder='Filter by prefix'
                value={keyPrefix}
                onChange={(event) => setKeyPrefix(event.target.value)}
                disabled={!selectedStore}
              />

              {!selectedStore && (
                <EmptyState
                  icon={KeyRound}
                  title='Select a store'
                  description='Pick a store to list keys.'
                />
              )}

              {selectedStore && keysQuery.isLoading && <Skeleton className='h-16 w-full' />}

              {selectedStore && !keysQuery.isLoading && keys.length === 0 && (
                <EmptyState
                  icon={KeyRound}
                  title='No keys yet'
                  description='Create a key to start storing values.'
                />
              )}

              {selectedStore && keys.length > 0 && (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Key</TableHead>
                      <TableHead>Expiration</TableHead>
                      <TableHead>Metadata</TableHead>
                      <TableHead />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {keys.map((keyInfo) => (
                      <TableRow
                        key={keyInfo.name}
                        data-state={selectedKey?.name === keyInfo.name ? 'selected' : undefined}
                      >
                        <TableCell>
                          <button
                            type='button'
                            className='text-left font-medium text-primary'
                            onClick={() => handleSelectKey(keyInfo)}
                          >
                            {keyInfo.name}
                          </button>
                        </TableCell>
                        <TableCell className='text-xs text-muted-foreground'>
                          {formatExpiration(keyInfo.expiration)}
                        </TableCell>
                        <TableCell className='text-xs text-muted-foreground'>
                          {keyInfo.metadata ? 'JSON' : '—'}
                        </TableCell>
                        <TableCell className='text-right'>
                          <Button
                            variant='ghost'
                            size='icon-sm'
                            onClick={() => handleDeleteKey(keyInfo)}
                            className='text-destructive hover:text-destructive'
                          >
                            <Trash2 className='h-4 w-4' />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Key editor</CardTitle>
              <CardDescription>
                {selectedStore
                  ? 'Create or update a key in the selected store.'
                  : 'Select a store to edit keys.'}
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-4'>
              <div className='grid gap-4 md:grid-cols-[1fr_180px]'>
                <Input
                  placeholder='Key name'
                  value={keyName}
                  onChange={(event) => setKeyName(event.target.value)}
                  disabled={!selectedStore}
                />
                <Input
                  placeholder='Expiration (unix seconds)'
                  value={keyExpiration}
                  onChange={(event) => setKeyExpiration(event.target.value)}
                  disabled={!selectedStore}
                />
              </div>

              <textarea
                className='min-h-[120px] w-full rounded-3xl border border-input bg-input/30 px-3 py-2 text-sm transition-colors outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50'
                placeholder='Value'
                value={keyValue}
                onChange={(event) => setKeyValue(event.target.value)}
                disabled={!selectedStore || valueQuery.isLoading}
              />

              <textarea
                className='min-h-[120px] w-full rounded-3xl border border-input bg-input/30 px-3 py-2 text-sm font-mono transition-colors outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50'
                placeholder='Metadata (JSON)'
                value={keyMetadata}
                onChange={(event) => setKeyMetadata(event.target.value)}
                disabled={!selectedStore}
              />

              <div className='flex flex-wrap gap-2'>
                <Button
                  onClick={handleSaveKey}
                  disabled={!selectedStore || setKeyMutation.isPending}
                >
                  Save key
                </Button>
                <Button variant='outline' onClick={handleClearEditor} disabled={!selectedStore}>
                  Clear
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
