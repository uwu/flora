import { getGuildLogs } from '@uwu/flora-api-client'
import { authApiOptions, authHeaders, expectOk } from '../lib/http'
import { logger } from '../lib/logger'
import { type LogEntry, printLogEntry, streamSseLogs } from '../lib/logs'
import type { CliConfig } from '../lib/types'

export async function logs(config: CliConfig, guild?: string, limit = 100): Promise<void> {
  if (!guild) {
    throw new Error('Missing guild; pass --guild <guild_id> to read logs')
  }

  const entries = await expectOk<LogEntry[]>(
    getGuildLogs({
      ...authApiOptions(config),
      path: { guild_id: guild },
      query: { limit }
    })
  )

  if (entries.length === 0) {
    logger.log('No logs found')
    return
  }

  for (const entry of entries) {
    printLogEntry(entry)
  }
}

export async function streamLogs(config: CliConfig, guild?: string): Promise<void> {
  if (!guild) {
    throw new Error('Missing guild; pass --guild <guild_id> to stream logs')
  }

  const headers = authHeaders(config)
  const streamPath = `/logs/${guild}/stream`
  const response = await fetch(`${config.apiUrl}${streamPath}`, { headers })

  if (!response.ok) {
    throw new Error(`Stream request failed: ${response.status} ${response.statusText}`)
  }

  logger.log('Streaming logs... (press Ctrl+C to stop)')
  await streamSseLogs(response, printLogEntry)
}
