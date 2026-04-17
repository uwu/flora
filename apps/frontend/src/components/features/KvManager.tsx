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
import {
  useCreateKvStoreMutation,
  useDeleteKvKeyMutation,
  useDeleteKvStoreMutation,
  useSetKvKeyMutation
} from '@/data/mutations'
import { useKvKeysQuery, useKvStoresQuery, useKvValueQuery } from '@/data/queries'
import { cn } from '@/lib/utils'
import type { KvStore, RawKvKeyInfo } from '@uwu/flora-api-client'
import { Database, KeyRound, Plus, RefreshCw, Trash2 } from 'lucide-react'
import { useMemo, useState } from 'react'

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
  const [keyValueDraft, setKeyValueDraft] = useState<string | null>(null)
  const [keyMetadata, setKeyMetadata] = useState('')
  const [keyExpiration, setKeyExpiration] = useState('')
  const [selectedKey, setSelectedKey] = useState<RawKvKeyInfo | null>(null)
  const [error, setError] = useState<string | null>(null)

  const storesQuery = useKvStoresQuery(guildId)

  const activeStore = useMemo(() => {
    const storeList = storesQuery.data ?? []
    if (selectedStore && storeList.some((store) => store.store_name === selectedStore)) {
      return selectedStore
    }
    return storeList[0]?.store_name ?? ''
  }, [selectedStore, storesQuery.data])

  const keysQuery = useKvKeysQuery(guildId, activeStore, keyPrefix)

  const valueQuery = useKvValueQuery(guildId, activeStore, selectedKey?.name ?? null)

  const createStoreMutation = useCreateKvStoreMutation({
    onSuccess: () => {
      setStoreName('')
      void storesQuery.refetch()
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to create store')
    }
  })

  const deleteStoreMutation = useDeleteKvStoreMutation({
    onSuccess: () => {
      void storesQuery.refetch()
      setSelectedStore('')
      resetEditor()
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to delete store')
    }
  })

  const setKeyMutation = useSetKvKeyMutation({
    onSuccess: () => {
      void keysQuery.refetch()
      setSelectedKey(null)
      setKeyName('')
      setKeyValueDraft('')
      setKeyMetadata('')
      setKeyExpiration('')
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to save key')
    }
  })

  const deleteKeyMutation = useDeleteKvKeyMutation({
    onSuccess: () => {
      void keysQuery.refetch()
      setSelectedKey(null)
      setKeyName('')
      setKeyValueDraft('')
      setKeyMetadata('')
      setKeyExpiration('')
    },
    onError: (err: any) => {
      setError(err.message || 'Failed to delete key')
    }
  })

  const stores = storesQuery.data ?? []
  const keys = keysQuery.data?.keys ?? []

  const resolvedKeyValue = keyValueDraft ?? valueQuery.data?.value ?? ''

  const resetEditor = () => {
    setSelectedKey(null)
    setKeyName('')
    setKeyValueDraft('')
    setKeyMetadata('')
    setKeyExpiration('')
  }

  const storeSummary = useMemo(() => {
    if (!activeStore) return 'Select a store to view keys.'
    if (!keysQuery.data) return 'Loading keys…'
    if (keysQuery.data.listComplete) return `${keys.length} keys loaded`
    return `${keys.length} keys loaded (more available)`
  }, [activeStore, keys.length, keysQuery.data])

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
      path: { guild_id: store.guild_id, store_name: store.store_name }
    })
  }

  const handleSelectKey = (keyInfo: RawKvKeyInfo) => {
    setSelectedKey(keyInfo)
    setKeyName(keyInfo.name)
    setKeyValueDraft(null)
    setKeyMetadata(formatMetadata(keyInfo.metadata))
    setKeyExpiration(keyInfo.expiration ? String(keyInfo.expiration) : '')
    setError(null)
  }

  const handleSaveKey = () => {
    setError(null)
    if (!activeStore) {
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
      path: {
        guild_id: guildId,
        store_name: activeStore,
        key: name
      },
      body: {
        value: resolvedKeyValue,
        metadata,
        expiration
      }
    })
  }

  const handleDeleteKey = (keyInfo: RawKvKeyInfo) => {
    setError(null)
    if (!activeStore) return
    if (!confirm(`Delete key ${keyInfo.name}?`)) return
    void deleteKeyMutation.mutateAsync({
      path: {
        guild_id: guildId,
        store_name: activeStore,
        key: keyInfo.name
      }
    })
  }

  const handleClearEditor = () => {
    setSelectedKey(null)
    setKeyName('')
    setKeyValueDraft('')
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
                    activeStore === store.store_name && 'border-primary/50 bg-primary/5'
                  )}
                >
                  <button
                    type='button'
                    className='flex-1 text-left font-medium'
                    onClick={() => {
                      setSelectedStore(store.store_name)
                      resetEditor()
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
                disabled={!activeStore || keysQuery.isFetching}
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
                disabled={!activeStore}
              />

              {!activeStore && (
                <EmptyState
                  icon={KeyRound}
                  title='Select a store'
                  description='Pick a store to list keys.'
                />
              )}

              {activeStore && keysQuery.isLoading && <Skeleton className='h-16 w-full' />}

              {activeStore && !keysQuery.isLoading && keys.length === 0 && (
                <EmptyState
                  icon={KeyRound}
                  title='No keys yet'
                  description='Create a key to start storing values.'
                />
              )}

              {activeStore && keys.length > 0 && (
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
                {activeStore
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
                  disabled={!activeStore}
                />
                <Input
                  placeholder='Expiration (unix seconds)'
                  value={keyExpiration}
                  onChange={(event) => setKeyExpiration(event.target.value)}
                  disabled={!activeStore}
                />
              </div>

              <textarea
                className='min-h-[120px] w-full rounded-3xl border border-input bg-input/30 px-3 py-2 text-sm transition-colors outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50'
                placeholder='Value'
                value={resolvedKeyValue}
                onChange={(event) => setKeyValueDraft(event.target.value)}
                disabled={!activeStore || valueQuery.isLoading}
              />

              <textarea
                className='min-h-[120px] w-full rounded-3xl border border-input bg-input/30 px-3 py-2 text-sm font-mono transition-colors outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50'
                placeholder='Metadata (JSON)'
                value={keyMetadata}
                onChange={(event) => setKeyMetadata(event.target.value)}
                disabled={!activeStore}
              />

              <div className='flex flex-wrap gap-2'>
                <Button onClick={handleSaveKey} disabled={!activeStore || setKeyMutation.isPending}>
                  Save key
                </Button>
                <Button variant='outline' onClick={handleClearEditor} disabled={!activeStore}>
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
