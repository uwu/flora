import { authHeaders, createApiClient, expectOk } from '../lib/http'
import { logger } from '../lib/logger'
import { type LogEntry, printLogEntry, streamSseLogs } from '../lib/logs'
import type { CliConfig } from '../lib/types'

export async function logs(config: CliConfig, guild?: string, limit = 100): Promise<void> {
  const client = createApiClient(config)

  const entries = guild
    ? await expectOk(
        client.GET('/logs/{guild_id}', {
          params: {
            path: { guild_id: guild },
            query: { limit }
          },
          headers: authHeaders(config)
        })
      )
    : await expectOk(
        client.GET('/logs', {
          params: { query: { limit } },
          headers: authHeaders(config)
        })
      )

  if (entries.length === 0) {
    logger.log('No logs found')
    return
  }

  for (const entry of entries as LogEntry[]) {
    printLogEntry(entry)
  }
}

export async function streamLogs(config: CliConfig, guild?: string): Promise<void> {
  const headers = authHeaders(config)
  const streamPath = guild ? `/logs/${guild}/stream` : '/logs/stream'
  const response = await fetch(`${config.apiUrl}${streamPath}`, { headers })

  if (!response.ok) {
    throw new Error(`Stream request failed: ${response.status} ${response.statusText}`)
  }

  logger.log('Streaming logs... (press Ctrl+C to stop)')
  await streamSseLogs(response, printLogEntry)
}
