import { $api } from '@/lib/openapi-client'

export const useCreateTokenMutation = (options?: any) =>
  $api.useMutation('post', '/tokens/', options as any)

export const useDeleteTokenMutation = (options?: any) =>
  $api.useMutation('delete', '/tokens/{token_id}', options as any)
