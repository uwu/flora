import { client as generatedClient } from './generated/client.gen'
import type { Config as OpenApiTsClientConfig } from './generated/client'
export * from './generated/index'
export * from './generated/@tanstack/react-query.gen'
export type { OpenApiTsClientConfig }

export const client = generatedClient

export function configureOpenApiClient(config: OpenApiTsClientConfig) {
  return client.setConfig(config)
}
