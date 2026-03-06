import { useCallback } from 'react'

import { redirectToLogin } from '@/lib/utils'

export function useLoginRedirect() {
  return useCallback(() => {
    redirectToLogin()
  }, [])
}
