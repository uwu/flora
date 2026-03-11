import { $api } from '@/lib/openapi-client'

const KEY_LIMIT = 200

export const useKvStoresQuery = (guildId: string) =>
  $api.useQuery(
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

export const useKvKeysQuery = (guildId: string, selectedStore: string, keyPrefix: string) =>
  $api.useQuery(
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

export const useKvValueQuery = (guildId: string, selectedStore: string, keyName: string | null) =>
  $api.useQuery(
    'get',
    '/kv/{guild_id}/{store_name}/{key}',
    {
      params: {
        path: {
          guild_id: guildId,
          store_name: selectedStore,
          key: keyName ?? ''
        }
      }
    },
    {
      enabled: !!guildId && !!selectedStore && !!keyName,
      refetchOnWindowFocus: false
    }
  )
