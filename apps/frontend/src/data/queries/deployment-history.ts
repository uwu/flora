import { useQuery } from '@tanstack/react-query'

import { requestJson } from '@/data/api/request'
import type { DeploymentRevision } from '@/data/deployments/types'

export const useDeploymentHistoryQuery = (guildId: string) =>
  useQuery({
    queryKey: ['deployment-history', guildId],
    queryFn: () =>
      requestJson<DeploymentRevision[]>(
        `/deployments/${encodeURIComponent(guildId)}/history?limit=100`
      )
  })

export const useDeploymentRevisionQuery = (
  guildId: string,
  revisionId: string | null,
  enabled: boolean
) => {
  const resolvedRevisionId = revisionId ?? ''
  return useQuery({
    queryKey: ['deployment-revision', guildId, revisionId],
    enabled,
    queryFn: () =>
      requestJson<DeploymentRevision>(
        `/deployments/${encodeURIComponent(guildId)}/revisions/${resolvedRevisionId}`
      )
  })
}
