import { useMutation, useQueryClient } from '@tanstack/react-query'
import {
  rollbackDeploymentHandlerMutation,
  type RollbackDeploymentHandlerResponse
} from '@uwu/flora-api-client'

type ApiQueryKey = {
  _id?: string
  path?: {
    guild_id?: string
  }
}

function isApiQueryKey(value: unknown): value is ApiQueryKey {
  return typeof value === 'object' && value !== null
}

export const useRollbackDeploymentMutation = (
  guildId: string,
  onSuccess?: (revision: RollbackDeploymentHandlerResponse) => void
) => {
  const queryClient = useQueryClient()

  return useMutation({
    ...rollbackDeploymentHandlerMutation(),
    onSuccess: async (revision) => {
      onSuccess?.(revision)

      await queryClient.invalidateQueries({
        predicate: (query) => {
          const queryKey = query.queryKey[0]
          if (!isApiQueryKey(queryKey)) return false
          if (queryKey.path?.guild_id !== guildId) return false
          return (
            queryKey._id === 'listDeploymentHistoryHandler' ||
            queryKey._id === 'getDeploymentRevisionHandler'
          )
        }
      })
    }
  })
}
