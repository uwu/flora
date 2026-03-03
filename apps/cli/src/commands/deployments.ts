import { loadProjectConfig } from '../lib/config'
import { authHeaders, createApiClient, expectOk } from '../lib/http'
import { logger } from '../lib/logger'
import { promptIfMissing } from '../lib/prompts'
import type { CliConfig } from '../lib/types'
import { zipProject } from '../lib/zip'

const BUILD_SSE_TIMEOUT = 60_000

export async function deploy(
  config: CliConfig,
  guildArg: string | undefined,
  entryArg: string | undefined,
  root?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const projectConfig = await loadProjectConfig()
  const entry = entryArg ?? projectConfig.entry ?? 'src/main.ts'
  const projectRoot = root ?? projectConfig.root ?? '.'

  logger.log('Uploading project...')
  const { zip, fileCount } = await zipProject(projectRoot)
  const zipSize = formatBytes(zip.byteLength)
  logger.log(`Uploading project... done (${fileCount} files, ${zipSize})`)

  const formData = new FormData()
  formData.append('guild_id', guild)
  formData.append('entry', entry)
  formData.append('project_zip', new Blob([zip as BlobPart]), 'project.zip')

  const baseUrl = config.apiUrl
  const headers = authHeaders(config)

  const createRes = await fetch(`${baseUrl}/builds`, {
    method: 'POST',
    headers,
    body: formData
  })

  if (!createRes.ok) {
    const body = await createRes.text().catch(() => '')
    throw new Error(`Build creation failed (${createRes.status}): ${body}`)
  }

  const { build_id, status } = (await createRes.json()) as { build_id: string; status: string }
  logger.log(`Building... (${build_id})`)

  // stream build logs via SSE
  const logsUrl = `${baseUrl}/builds/${build_id}/logs`
  const finished = await streamBuildLogs(logsUrl, headers, BUILD_SSE_TIMEOUT)

  if (!finished) {
    logger.log(`Build still running: ${build_id}`)
    logger.log(`Run \`flora builds tail ${build_id}\` to follow logs.`)
    return
  }

  // fetch final build status
  const buildRes = await fetch(`${baseUrl}/builds/${build_id}`, { headers })
  if (!buildRes.ok) {
    throw new Error(`Failed to fetch build result: ${buildRes.status}`)
  }

  const build = (await buildRes.json()) as {
    status: string
    guild_id: string
    entry: string
    error?: string
  }

  if (build.status === 'failed') {
    throw new Error(`Build failed: ${build.error ?? 'unknown error'}`)
  }

  logger.log(`\nDeployed guild ${build.guild_id}`)
  logger.log(`  entry: ${build.entry}`)
  logger.log(`  updated: ${new Date().toISOString()}`)
}

async function streamBuildLogs(
  url: string,
  headers: Record<string, string>,
  timeout: number
): Promise<boolean> {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), timeout)

  try {
    const res = await fetch(url, { headers, signal: controller.signal })
    if (!res.ok || !res.body) {
      return false
    }

    const reader = res.body.getReader()
    const decoder = new TextDecoder()
    let buffer = ''

    while (true) {
      const { value, done } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })

      for (;;) {
        const eventEnd = buffer.indexOf('\n\n')
        if (eventEnd < 0) break

        const event = buffer.slice(0, eventEnd)
        buffer = buffer.slice(eventEnd + 2)

        for (const line of event.split('\n')) {
          if (line.startsWith('event: done')) {
            return true
          }
          if (line.startsWith('data: ')) {
            const data = line.slice(6)
            logger.log(`  ↳ ${data}`)
          }
        }
      }
    }

    return true
  } catch (err) {
    if (err instanceof DOMException && err.name === 'AbortError') {
      return false
    }
    throw err
  } finally {
    clearTimeout(timer)
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)}kb`
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`
}

export async function get(config: CliConfig, guildArg: string | undefined): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')

  const client = createApiClient(config)
  const deployment = await expectOk(
    client.GET('/deployments/{guild_id}', {
      params: { path: { guild_id: guild } },
      headers: authHeaders(config)
    })
  )

  logger.log(
    `Guild ${deployment.guild_id}\n  entry: ${deployment.entry}\n  created: ${deployment.created_at}\n  updated: ${deployment.updated_at}`
  )
}

export async function list(config: CliConfig): Promise<void> {
  const client = createApiClient(config)
  const deployments = await expectOk(
    client.GET('/deployments/', {
      headers: authHeaders(config)
    })
  )

  if (deployments.length === 0) {
    logger.log('No deployments found')
    return
  }

  for (const deployment of deployments) {
    logger.log(
      `${deployment.guild_id} entry=${deployment.entry} created=${deployment.created_at} updated=${deployment.updated_at}`
    )
  }
}

export async function health(config: CliConfig): Promise<void> {
  const client = createApiClient(config)
  const response = await expectOk(
    client.GET('/health/', {
      headers: authHeaders(config),
      parseAs: 'text'
    })
  )

  logger.log(`${response}`)
}
