const STATIC_CACHE = 'flora-static-v1'
const IMAGE_CACHE = 'flora-images-v1'

self.addEventListener('install', (event) => {
  event.waitUntil(self.skipWaiting())
})

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      const cacheNames = await caches.keys()
      for (const name of cacheNames) {
        if (name !== STATIC_CACHE && name !== IMAGE_CACHE) {
          await caches.delete(name)
        }
      }
      await self.clients.claim()
    })()
  )
})

self.addEventListener('fetch', (event) => {
  const { request } = event
  if (request.method !== 'GET') return

  const url = new URL(request.url)

  if (url.hostname === 'cdn.discordapp.com') {
    event.respondWith(staleWhileRevalidate(event, request, IMAGE_CACHE))
    return
  }

  if (url.origin !== self.location.origin) return
  if (url.pathname.startsWith('/api')) return

  if (url.pathname.startsWith('/assets/')) {
    event.respondWith(cacheFirst(request, STATIC_CACHE))
    return
  }

  if (isHtmlRequest(request, url)) {
    event.respondWith(networkFirst(request, STATIC_CACHE))
  }
})

function isHtmlRequest(request, url) {
  const acceptHeader = request.headers.get('accept') || ''
  if (acceptHeader.includes('text/html')) return true
  if (url.pathname === '/') return true
  return !url.pathname.includes('.')
}

async function cacheFirst(request, cacheName) {
  const cache = await caches.open(cacheName)
  const cached = await cache.match(request)
  if (cached) return cached
  const response = await fetch(request)
  if (response && (response.ok || response.type === 'opaque')) {
    await cache.put(request, response.clone())
  }
  return response
}

async function networkFirst(request, cacheName) {
  const cache = await caches.open(cacheName)
  try {
    const response = await fetch(request)
    if (response && (response.ok || response.type === 'opaque')) {
      await cache.put(request, response.clone())
    }
    return response
  } catch (error) {
    const cached = await cache.match(request)
    if (cached) return cached
    return new Response('Offline', { status: 503 })
  }
}

async function staleWhileRevalidate(event, request, cacheName) {
  const cache = await caches.open(cacheName)
  const cached = await cache.match(request)
  const fetchPromise = fetch(request)
    .then((response) => {
      if (response && (response.ok || response.type === 'opaque')) {
        cache.put(request, response.clone())
      }
      return response
    })
    .catch(() => null)

  event.waitUntil(fetchPromise)

  if (cached) return cached
  const response = await fetchPromise
  if (response) return response
  return new Response('', { status: 504 })
}
