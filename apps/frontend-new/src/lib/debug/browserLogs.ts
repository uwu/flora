const endpoint = '/__debug/browser-log'

function serialize(value: unknown): string {
  if (value instanceof Error) return `${value.name}: ${value.message}\n${value.stack ?? ''}`
  if (typeof value === 'string') return value

  try {
    return JSON.stringify(value)
  } catch {
    return String(value)
  }
}

function send(level: string, values: unknown[]) {
  if (!import.meta.env.DEV) return

  const payload = JSON.stringify({
    level,
    url: window.location.href,
    timestamp: new Date().toISOString(),
    message: values.map(serialize).join(' ')
  })

  if (!navigator.sendBeacon(endpoint, payload)) {
    void fetch(endpoint, {
      method: 'POST',
      body: payload,
      keepalive: true,
      headers: { 'content-type': 'application/json' }
    }).catch(() => {})
  }
}

export function installBrowserLogForwarding() {
  if (!import.meta.env.DEV) return

  for (const level of ['debug', 'info', 'log', 'warn', 'error'] as const) {
    const original = console[level]
    console[level] = (...values: unknown[]) => {
      send(level, values)
      original(...values)
    }
  }

  window.addEventListener('error', (event) => {
    send('error', [event.error ?? event.message])
  })

  window.addEventListener('unhandledrejection', (event) => {
    send('error', ['Unhandled promise rejection', event.reason])
  })
}
