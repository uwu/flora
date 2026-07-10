export type FeatureAuthorizationRequest = {
  feature: string
  userId: string
  metadata?: unknown
}

export type OrchestratorDefinition = {
  authorizeFeature: (request: FeatureAuthorizationRequest) => boolean | Promise<boolean>
}

declare global {
  var __floraRuntimeKind: 'orchestrator' | undefined
  var __floraAuthorizeFeature:
    | ((request: FeatureAuthorizationRequest) => Promise<boolean>)
    | undefined
}

/** Register the policy function used by Flora's trusted singleton orchestrator runtime. */
export function defineOrchestrator(definition: OrchestratorDefinition): void {
  if (globalThis.__floraRuntimeKind !== 'orchestrator') {
    throw new Error('defineOrchestrator can only be used by the trusted orchestrator deployment')
  }
  if (typeof definition?.authorizeFeature !== 'function') {
    throw new TypeError('defineOrchestrator requires an authorizeFeature function')
  }
  if (globalThis.__floraAuthorizeFeature) {
    throw new Error('defineOrchestrator may only be called once')
  }

  globalThis.__floraAuthorizeFeature = async (request) =>
    Boolean(await definition.authorizeFeature(request))
}
