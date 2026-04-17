import { useQuery } from '@tanstack/react-query'
import {
  getValueHandlerOptions,
  listKeysHandlerOptions,
  listStoresHandlerOptions
} from '@uwu/flora-api-client'

const KEY_LIMIT = 200

export const useKvStoresQuery = (guildId: string) =>
  useQuery({
    ...listStoresHandlerOptions({
      query: {
        guild_id: guildId
      }
    }),
    enabled: !!guildId,
    refetchOnWindowFocus: false
  })

export const useKvKeysQuery = (guildId: string, selectedStore: string, keyPrefix: string) =>
  useQuery({
    ...listKeysHandlerOptions({
      path: { guild_id: guildId, store_name: selectedStore },
      query: {
        prefix: keyPrefix.trim() ? keyPrefix.trim() : undefined,
        limit: KEY_LIMIT
      }
    }),
    enabled: !!guildId && !!selectedStore,
    refetchOnWindowFocus: false
  })

export const useKvValueQuery = (guildId: string, selectedStore: string, keyName: string | null) =>
  useQuery({
    ...getValueHandlerOptions({
      path: {
        guild_id: guildId,
        store_name: selectedStore,
        key: keyName ?? ''
      }
    }),
    enabled: !!guildId && !!selectedStore && !!keyName,
    refetchOnWindowFocus: false
  })
