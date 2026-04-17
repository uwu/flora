import { useQuery } from '@tanstack/react-query'
import {
  getDeploymentRevisionHandlerOptions,
  listDeploymentHistoryHandlerOptions
} from '@uwu/flora-api-client'

const HISTORY_LIMIT = 100

export const useDeploymentHistoryQuery = (guildId: string) =>
  useQuery({
    ...listDeploymentHistoryHandlerOptions({
      path: {
        guild_id: guildId
      },
      query: {
        limit: HISTORY_LIMIT
      }
    }),
    enabled: !!guildId
  })

export const useDeploymentRevisionQuery = (
  guildId: string,
  revisionId: string | null,
  enabled: boolean
) => {
  const resolvedRevisionId = revisionId ?? ''

  return useQuery({
    ...getDeploymentRevisionHandlerOptions({
      path: {
        guild_id: guildId,
        revision_id: resolvedRevisionId
      }
    }),
    enabled
  })
}
