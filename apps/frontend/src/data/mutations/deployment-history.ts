import { useMutation, useQueryClient } from '@tanstack/react-query'

import { requestJson } from '@/data/api/request'
import type { DeploymentRevision } from '@/data/deployments/types'

export const useRollbackDeploymentMutation = (
  guildId: string,
  onSuccess?: (revision: DeploymentRevision) => void
) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (revisionId: string) =>
      requestJson<DeploymentRevision>(
        `/deployments/${encodeURIComponent(guildId)}/rollback/${revisionId}`,
        {
          method: 'POST'
        }
      ),
    onSuccess: async (revision) => {
      onSuccess?.(revision)
      await queryClient.invalidateQueries({ queryKey: ['deployment-history', guildId] })
      await queryClient.invalidateQueries({ queryKey: ['deployment-revision', guildId] })
    }
  })
}
