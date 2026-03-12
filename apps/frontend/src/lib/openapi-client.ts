import { QueryClient } from '@tanstack/react-query'
import { type $defs, createApiClient, type webhooks } from '@uwu/flora-api-client'
import createRQClient from 'openapi-react-query'

// Use Vite env override when present; default to the dev proxy at /api.
const baseUrl = import.meta.env.VITE_API_BASE ?? '/api'

function trimTrailingSlashFromPath(url: string) {
  return url.replace(/\/(?=($|[?#]))/, '')
}

function normalizeInput(input: Parameters<typeof fetch>[0]): Parameters<typeof fetch>[0] {
  if (typeof input === 'string') {
    return trimTrailingSlashFromPath(input)
  }

  if (input instanceof URL) {
    if (input.pathname.length > 1 && input.pathname.endsWith('/')) {
      input.pathname = input.pathname.slice(0, -1)
    }
    return input
  }

  const normalizedUrl = trimTrailingSlashFromPath(input.url)
  return normalizedUrl === input.url ? input : new Request(normalizedUrl, input)
}

const fetchWithCreds: typeof fetch = (input, init) =>
  fetch(normalizeInput(input), { credentials: 'include', ...init })

type _OpenApiRootTypes = webhooks | $defs
const _openApiTypeMarker: _OpenApiRootTypes | null = null
void _openApiTypeMarker

export const queryClient = new QueryClient()

const fetchClient = createApiClient({ apiUrl: baseUrl, fetch: fetchWithCreds })

export const $api = createRQClient(fetchClient)
