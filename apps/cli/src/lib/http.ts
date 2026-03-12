import { createApiClient } from '@uwu/flora-api-client'

import type { CliConfig } from './types'

export { createApiClient }

export function authHeaders(config: CliConfig): Record<string, string> {
  if (!config.token) {
    throw new Error('Missing API token; run `flora login --token <token>`')
  }

  return {
    authorization: `Bearer ${config.token}`
  }
}

export async function expectOk<T>(
  promise: Promise<{ data?: T; error?: unknown; response: Response }>
): Promise<T> {
  const { data, error, response } = await promise
  if (!response.ok) {
    if (
      error && typeof error === 'object' && 'message' in error && typeof error.message === 'string'
    ) {
      throw new Error(error.message)
    }
    throw new Error(`Request failed: ${response.status} ${response.statusText}`)
  }

  return data as T
}
