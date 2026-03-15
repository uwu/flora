import { logger } from './logger'

export type LogEntry = {
  timestamp: number
  level: string
  target: string
  guild_id?: string | null
  message: string
}

export function printLogEntry(entry: LogEntry): void {
  const dt = new Date(entry.timestamp)
  const timestamp = Number.isNaN(dt.getTime())
    ? String(entry.timestamp)
    : dt.toISOString().replace('T', ' ').replace('Z', '')

  const level = colorLevel(entry.level)
  const guild = entry.guild_id ?? '-'

  logger.log(`${timestamp} ${level} [${guild}] ${entry.target}: ${entry.message}`)
}

function colorLevel(level: string): string {
  switch (level) {
    case 'error':
      return '\x1b[31mERROR\x1b[0m'
    case 'warn':
      return '\x1b[33mWARN\x1b[0m'
    case 'info':
      return '\x1b[32mINFO\x1b[0m'
    case 'debug':
      return '\x1b[34mDEBUG\x1b[0m'
    case 'trace':
      return '\x1b[90mTRACE\x1b[0m'
    default:
      return level
  }
}

export async function streamSseLogs(
  response: Response,
  onLog: (entry: LogEntry) => void
): Promise<void> {
  if (!response.body) {
    throw new Error('SSE stream missing response body')
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''

  while (true) {
    const { value, done } = await reader.read()
    if (done) {
      break
    }

    buffer += decoder.decode(value, { stream: true })

    for (;;) {
      const eventEnd = buffer.indexOf('\n\n')
      if (eventEnd < 0) {
        break
      }

      const event = buffer.slice(0, eventEnd)
      buffer = buffer.slice(eventEnd + 2)

      for (const line of event.split('\n')) {
        if (!line.startsWith('data: ')) {
          continue
        }

        const raw = line.slice(6)
        try {
          const parsed = JSON.parse(raw) as LogEntry
          onLog(parsed)
        } catch {
          // Ignore malformed events.
        }
      }
    }
  }
}
