import { $api } from '@/lib/openapi-client'

export const useDeploymentQuery = (guildId?: string) =>
  $api.useQuery(
    'get',
    '/deployments/{guild_id}',
    {
      params: {
        path: { guild_id: guildId ?? '' },
        query: { include_bundle: true }
      }
    },
    {
      enabled: !!guildId
    }
  )

export const useLogsQuery = (guildId?: string) =>
  $api.useQuery(
    'get',
    '/logs/{guild_id}',
    {
      params: {
        path: { guild_id: guildId ?? '' },
        query: { limit: 100 }
      }
    },
    {
      enabled: !!guildId,
      refetchInterval: 3000
    }
  )
