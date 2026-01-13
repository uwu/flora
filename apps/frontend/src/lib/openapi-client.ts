import createClient from 'openapi-fetch'
import createRQClient from 'openapi-react-query'
import type { paths } from './openapi-schema'

// Use Vite env override when present; default to the dev proxy at /api.
const baseUrl = import.meta.env.VITE_API_BASE ?? '/api'

const fetchWithCreds: typeof fetch = (input, init) =>
  fetch(input, { credentials: 'include', ...init })

export const api = createClient<paths>({ baseUrl, fetch: fetchWithCreds })

export const { client: rqClient, withClient } = createRQClient(api)
