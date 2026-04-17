import { QueryClient } from '@tanstack/react-query'
import { client, configureOpenApiClient } from '@uwu/flora-api-client'

type ApiError = Error & {
  cause?: unknown
  status?: number
}

type ClientWithErrorInterceptorFlag = typeof client & {
  __floraErrorInterceptorInstalled?: boolean
}

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

function toApiError(error: unknown, status?: number): ApiError {
  if (error instanceof Error) {
    const apiError = error as ApiError
    apiError.status = status
    return apiError
  }

  const message =
    typeof error === 'string'
      ? error
      : typeof error === 'object' &&
          error !== null &&
          'message' in error &&
          typeof error.message === 'string'
        ? error.message
        : status
          ? `Request failed (${status})`
          : 'Request failed'

  const apiError = new Error(message) as ApiError
  apiError.status = status
  apiError.cause = error
  return apiError
}

const fetchWithCreds: typeof fetch = (input, init) =>
  fetch(normalizeInput(input), { credentials: 'include', ...init })

configureOpenApiClient({
  baseUrl,
  fetch: fetchWithCreds
})

const clientWithFlags = client as ClientWithErrorInterceptorFlag

if (!clientWithFlags.__floraErrorInterceptorInstalled) {
  client.interceptors.error.use((error, response) => toApiError(error, response?.status))
  clientWithFlags.__floraErrorInterceptorInstalled = true
}

export const queryClient = new QueryClient()
