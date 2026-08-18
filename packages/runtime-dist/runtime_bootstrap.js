import { core } from 'ext:core/mod.js'

core.loadExtScript('ext:deno_webidl/00_webidl.js')
core.loadExtScript('ext:deno_web/00_infra.js')
core.loadExtScript('ext:deno_web/01_dom_exception.js')
core.loadExtScript('ext:deno_web/01_mimesniff.js')
core.loadExtScript('ext:deno_web/01_urlpattern.js')
core.loadExtScript('ext:deno_web/01_console.js')
core.loadExtScript('ext:deno_web/01_broadcast_channel.js')
core.loadExtScript('ext:deno_web/02_event.js')
core.loadExtScript('ext:deno_web/02_structured_clone.js')
core.loadExtScript('ext:deno_web/02_timers.js')
core.loadExtScript('ext:deno_web/03_abort_signal.js')
core.loadExtScript('ext:deno_web/04_global_interfaces.js')
core.loadExtScript('ext:deno_web/05_base64.js')
core.loadExtScript('ext:deno_web/06_streams.js')
core.loadExtScript('ext:deno_web/08_text_encoding.js')
core.loadExtScript('ext:deno_web/09_file.js')
core.loadExtScript('ext:deno_web/10_filereader.js')
core.loadExtScript('ext:deno_web/12_location.js')
core.loadExtScript('ext:deno_web/13_message_port.js')
core.loadExtScript('ext:deno_web/14_compression.js')
core.loadExtScript('ext:deno_web/15_performance.js')
core.loadExtScript('ext:deno_web/16_image_data.js')

core.loadExtScript('ext:deno_net/01_net.js')
core.loadExtScript('ext:deno_net/02_tls.js')

const headers = core.loadExtScript('ext:deno_fetch/20_headers.js')
const formData = core.loadExtScript('ext:deno_fetch/21_formdata.js')
core.loadExtScript('ext:deno_fetch/22_body.js')
core.loadExtScript('ext:deno_fetch/22_http_client.js')
const request = core.loadExtScript('ext:deno_fetch/23_request.js')
const response = core.loadExtScript('ext:deno_fetch/23_response.js')
const fetch = core.loadExtScript('ext:deno_fetch/26_fetch.js')
core.loadExtScript('ext:deno_fetch/27_eventsource.js')

core.loadExtScript('ext:deno_telemetry/telemetry.ts')
core.loadExtScript('ext:deno_telemetry/util.ts')

core.setWasmStreamingCallback(fetch.handleWasmStreaming)

Object.defineProperty(globalThis, 'fetch', {
  value: fetch.fetch,
  enumerable: true,
  configurable: true,
  writable: true
})

Object.defineProperty(globalThis, 'Request', {
  value: request.Request,
  enumerable: false,
  configurable: true,
  writable: true
})

Object.defineProperty(globalThis, 'Response', {
  value: response.Response,
  enumerable: false,
  configurable: true,
  writable: true
})

Object.defineProperty(globalThis, 'Headers', {
  value: headers.Headers,
  enumerable: false,
  configurable: true,
  writable: true
})

Object.defineProperty(globalThis, 'FormData', {
  value: formData.FormData,
  enumerable: false,
  configurable: true,
  writable: true
})
