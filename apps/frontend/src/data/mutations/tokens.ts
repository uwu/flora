import { useMutation } from '@tanstack/react-query'
import { createTokenHandlerMutation, deleteTokenHandlerMutation } from '@uwu/flora-api-client'

type CreateTokenMutationOptions = Omit<ReturnType<typeof createTokenHandlerMutation>, 'mutationFn'>
type DeleteTokenMutationOptions = Omit<ReturnType<typeof deleteTokenHandlerMutation>, 'mutationFn'>

export const useCreateTokenMutation = (options?: CreateTokenMutationOptions) =>
  useMutation({
    ...createTokenHandlerMutation(),
    ...options
  })

export const useDeleteTokenMutation = (options?: DeleteTokenMutationOptions) =>
  useMutation({
    ...deleteTokenHandlerMutation(),
    ...options
  })
