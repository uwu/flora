import { useQuery } from '@tanstack/react-query'
import { getDeploymentHandlerOptions, getGuildLogsOptions } from '@uwu/flora-api-client'

export const useDeploymentQuery = (guildId?: string) =>
  useQuery({
    ...getDeploymentHandlerOptions({
      path: { guild_id: guildId ?? '' },
      query: { include_bundle: true }
    }),
    enabled: !!guildId
  })

export const useLogsQuery = (guildId?: string) =>
  useQuery({
    ...getGuildLogsOptions({
      path: { guild_id: guildId ?? '' },
      query: { limit: 100 }
    }),
    enabled: !!guildId,
    refetchInterval: 3000
  })
