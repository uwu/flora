/* dprint-ignore-file
Bundled module map for multi-file deployments.
We compile all user files into a single script and keep a tiny in-memory
loader so ESM imports/exports work without a runtime filesystem. */
const __modules = {}
const __cache = {}
function __define(id, factory) {
  __modules[id] = factory
}
function __require(id) {
  if (__cache[id]) return __cache[id].exports
  const factory = __modules[id]
  if (!factory) throw new Error(`Module not found: ${id}`)
  const module = { exports: {} }
  __cache[id] = module
  factory(module.exports, module)
  return module.exports
}
