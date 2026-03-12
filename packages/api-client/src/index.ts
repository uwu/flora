import createClient from 'openapi-fetch'

import type { paths } from './generated/openapi-schema'

export type { $defs, components, operations, paths, webhooks } from './generated/openapi-schema'

export interface ApiClientConfig {
  apiUrl: string
  fetch?: typeof fetch
}

export function createApiClient(config: ApiClientConfig) {
  return createClient<paths>({
    baseUrl: config.apiUrl,
    fetch: config.fetch
  })
}
