import { core } from 'ext:core/mod.js'
import 'ext:deno_webidl/00_webidl.js'
import 'ext:deno_web/00_infra.js'
import * as domException from 'ext:deno_web/01_dom_exception.js'
import 'ext:deno_web/01_mimesniff.js'
import * as urlPattern from 'ext:deno_web/01_urlpattern.js'
import 'ext:deno_web/01_console.js'
import * as broadcastChannel from 'ext:deno_web/01_broadcast_channel.js'
import * as event from 'ext:deno_web/02_event.js'
import * as structuredCloneApi from 'ext:deno_web/02_structured_clone.js'
import * as timers from 'ext:deno_web/02_timers.js'
import * as abortSignal from 'ext:deno_web/03_abort_signal.js'
import 'ext:deno_web/04_global_interfaces.js'
import * as base64 from 'ext:deno_web/05_base64.js'
import * as streams from 'ext:deno_web/06_streams.js'
import * as textEncoding from 'ext:deno_web/08_text_encoding.js'
import * as file from 'ext:deno_web/09_file.js'
import * as fileReader from 'ext:deno_web/10_filereader.js'
import 'ext:deno_web/12_location.js'
import * as url from 'ext:deno_web/00_url.js'
import * as messagePort from 'ext:deno_web/13_message_port.js'
import * as compression from 'ext:deno_web/14_compression.js'
import * as performanceApi from 'ext:deno_web/15_performance.js'
import * as imageData from 'ext:deno_web/16_image_data.js'

import 'ext:deno_net/01_net.js'
import 'ext:deno_net/02_tls.js'

import * as headers from 'ext:deno_fetch/20_headers.js'
import * as formData from 'ext:deno_fetch/21_formdata.js'
import 'ext:deno_fetch/22_body.js'
import 'ext:deno_fetch/22_http_client.js'
import * as request from 'ext:deno_fetch/23_request.js'
import * as response from 'ext:deno_fetch/23_response.js'
import * as fetch from 'ext:deno_fetch/26_fetch.js'
import 'ext:deno_fetch/27_eventsource.js'

import 'ext:deno_telemetry/telemetry.ts'
import 'ext:deno_telemetry/util.ts'

core.setWasmStreamingCallback(fetch.handleWasmStreaming)

function defineGlobal(name, value, enumerable = false) {
  Object.defineProperty(globalThis, name, {
    value,
    enumerable,
    configurable: true,
    writable: true
  })
}

defineGlobal('fetch', fetch.fetch, true)
defineGlobal('Request', request.Request)
defineGlobal('Response', response.Response)
defineGlobal('Headers', headers.Headers)
defineGlobal('FormData', formData.FormData)
defineGlobal('ReadableStream', streams.ReadableStream)
defineGlobal('ReadableStreamDefaultReader', streams.ReadableStreamDefaultReader)
defineGlobal('ReadableStreamBYOBReader', streams.ReadableStreamBYOBReader)
defineGlobal('ReadableStreamBYOBRequest', streams.ReadableStreamBYOBRequest)
defineGlobal('ReadableByteStreamController', streams.ReadableByteStreamController)
defineGlobal('ReadableStreamDefaultController', streams.ReadableStreamDefaultController)
defineGlobal('WritableStream', streams.WritableStream)
defineGlobal('WritableStreamDefaultWriter', streams.WritableStreamDefaultWriter)
defineGlobal('WritableStreamDefaultController', streams.WritableStreamDefaultController)
defineGlobal('TransformStream', streams.TransformStream)
defineGlobal('TransformStreamDefaultController', streams.TransformStreamDefaultController)
defineGlobal('ByteLengthQueuingStrategy', streams.ByteLengthQueuingStrategy)
defineGlobal('CountQueuingStrategy', streams.CountQueuingStrategy)
defineGlobal('TextEncoder', textEncoding.TextEncoder)
defineGlobal('TextDecoder', textEncoding.TextDecoder)
defineGlobal('TextEncoderStream', textEncoding.TextEncoderStream)
defineGlobal('TextDecoderStream', textEncoding.TextDecoderStream)
defineGlobal('Blob', file.Blob)
defineGlobal('File', file.File)
defineGlobal('FileReader', fileReader.FileReader)
defineGlobal('URL', url.URL)
defineGlobal('URLSearchParams', url.URLSearchParams)
defineGlobal('URLPattern', urlPattern.URLPattern)
defineGlobal('AbortController', abortSignal.AbortController)
defineGlobal('AbortSignal', abortSignal.AbortSignal)
defineGlobal('Event', event.Event)
defineGlobal('EventTarget', event.EventTarget)
defineGlobal('CustomEvent', event.CustomEvent)
defineGlobal('MessageEvent', event.MessageEvent)
defineGlobal('CloseEvent', event.CloseEvent)
defineGlobal('ErrorEvent', event.ErrorEvent)
defineGlobal('ProgressEvent', event.ProgressEvent)
defineGlobal('PromiseRejectionEvent', event.PromiseRejectionEvent)
defineGlobal('BroadcastChannel', broadcastChannel.BroadcastChannel)
defineGlobal('CompressionStream', compression.CompressionStream)
defineGlobal('DecompressionStream', compression.DecompressionStream)
defineGlobal('MessageChannel', messagePort.MessageChannel)
defineGlobal('MessagePort', messagePort.MessagePort)
defineGlobal('ImageData', imageData.ImageData)
defineGlobal('DOMException', domException.DOMException)
defineGlobal('setTimeout', timers.setTimeout, true)
defineGlobal('setInterval', timers.setInterval, true)
defineGlobal('clearTimeout', timers.clearTimeout, true)
defineGlobal('clearInterval', timers.clearInterval, true)
defineGlobal(
  'structuredClone',
  messagePort.structuredClone ?? structuredCloneApi.structuredClone,
  true
)
defineGlobal('reportError', event.reportError, true)
defineGlobal('atob', base64.atob, true)
defineGlobal('btoa', base64.btoa, true)
defineGlobal('performance', performanceApi.performance, true)

performanceApi.setTimeOrigin()
