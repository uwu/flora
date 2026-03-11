import { $api } from '@/lib/openapi-client'

export const useDeployMutation = () => $api.useMutation('post', '/deployments/{guild_id}')
