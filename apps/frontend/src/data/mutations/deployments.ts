import { useMutation } from '@tanstack/react-query'
import { upsertDeploymentHandlerMutation } from '@uwu/flora-api-client'

type DeployMutationOptions = Omit<ReturnType<typeof upsertDeploymentHandlerMutation>, 'mutationFn'>

export const useDeployMutation = (options?: DeployMutationOptions) =>
  useMutation({
    ...upsertDeploymentHandlerMutation(),
    ...options
  })
