import { useMutation } from '@tanstack/react-query'
import {
  createStoreHandlerMutation,
  deleteKeyHandlerMutation,
  deleteStoreHandlerMutation,
  setValueHandlerMutation
} from '@uwu/flora-api-client'

type CreateKvStoreMutationOptions = Omit<
  ReturnType<typeof createStoreHandlerMutation>,
  'mutationFn'
>
type DeleteKvStoreMutationOptions = Omit<
  ReturnType<typeof deleteStoreHandlerMutation>,
  'mutationFn'
>
type SetKvKeyMutationOptions = Omit<ReturnType<typeof setValueHandlerMutation>, 'mutationFn'>
type DeleteKvKeyMutationOptions = Omit<ReturnType<typeof deleteKeyHandlerMutation>, 'mutationFn'>

export const useCreateKvStoreMutation = (options?: CreateKvStoreMutationOptions) =>
  useMutation({
    ...createStoreHandlerMutation(),
    ...options
  })

export const useDeleteKvStoreMutation = (options?: DeleteKvStoreMutationOptions) =>
  useMutation({
    ...deleteStoreHandlerMutation(),
    ...options
  })

export const useSetKvKeyMutation = (options?: SetKvKeyMutationOptions) =>
  useMutation({
    ...setValueHandlerMutation(),
    ...options
  })

export const useDeleteKvKeyMutation = (options?: DeleteKvKeyMutationOptions) =>
  useMutation({
    ...deleteKeyHandlerMutation(),
    ...options
  })
