import createClient from 'openapi-fetch'

import type { paths } from '../generated/openapi-schema'
import type { CliConfig } from './types'

export function normalizeUrl(raw: string): string {
  const url = new URL(raw)

  if (url.pathname.endsWith('/') && url.pathname.length > 1) {
    url.pathname = url.pathname.slice(0, -1)
  }

  if (url.pathname.includes('/api/kv/api/kv')) {
    url.pathname = url.pathname.replace('/api/kv/api/kv', '/api/kv')
  }

  return url.toString()
}

const normalizedFetch: typeof fetch = (input, init) => {
  if (typeof input === 'string') {
    return fetch(normalizeUrl(input), init)
  }
  if (input instanceof URL) {
    return fetch(new URL(normalizeUrl(input.toString())), init)
  }

  return fetch(new Request(normalizeUrl(input.url), input), init)
}

export function createApiClient(config: CliConfig) {
  return createClient<paths>({
    baseUrl: config.apiUrl,
    fetch: normalizedFetch
  })
}

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
