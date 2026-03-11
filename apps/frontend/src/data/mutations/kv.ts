import { $api } from '@/lib/openapi-client'

export const useCreateKvStoreMutation = (options?: any) =>
  $api.useMutation('post', '/kv/stores', options as any)

export const useDeleteKvStoreMutation = (options?: any) =>
  $api.useMutation('delete', '/kv/stores/{guild_id}/{store_name}', options as any)

export const useSetKvKeyMutation = (options?: any) =>
  $api.useMutation('put', '/kv/{guild_id}/{store_name}/{key}', options as any)

export const useDeleteKvKeyMutation = (options?: any) =>
  $api.useMutation('delete', '/kv/{guild_id}/{store_name}/{key}', options as any)
