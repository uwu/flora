import createClient from 'openapi-fetch'

import type { paths } from './generated/openapi-schema'

export type { $defs, components, operations, paths, webhooks } from './generated/openapi-schema'

export interface ApiClientConfig {
  apiUrl: string
}

export function createApiClient(config: ApiClientConfig) {
  return createClient<paths>({
    baseUrl: config.apiUrl
  })
}
