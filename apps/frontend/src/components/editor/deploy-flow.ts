import { createZipFromFiles } from './editor-utils'
import type { DeploymentFileRecord } from './types'

const API_BASE = import.meta.env.VITE_API_BASE ?? '/api'
const BUILD_SSE_TIMEOUT = 60_000

type BuildState = {
  status: string
  guild_id: string
  entry: string
  error?: string
  artifact?: { bundle: string; source_map?: string }
}

type CompletedBuild = Omit<BuildState, 'status' | 'artifact'> & {
  status: 'done'
  artifact: { bundle: string; source_map?: string }
}

export async function runEditorBuildFlow(args: {
  guildId: string
  fileContents: DeploymentFileRecord
  preferredEntry: string
  fallbackEntry: string
  onBuildLog: (line: string) => void
}) {
  const { guildId, fileContents, preferredEntry, fallbackEntry, onBuildLog } = args
  const { zip, fileNames } = createZipFromFiles(fileContents)

  const entry =
    fileContents[preferredEntry] != null
      ? preferredEntry
      : fileContents['src/main.ts'] != null
        ? 'src/main.ts'
        : fallbackEntry

  const formData = new FormData()
  formData.append('guild_id', guildId)
  formData.append('entry', entry)
  formData.append('project_zip', new Blob([zip as BlobPart]), 'project.zip')

  const createRes = await fetch(`${API_BASE}/builds`, {
    method: 'POST',
    credentials: 'include',
    body: formData
  })

  if (!createRes.ok) {
    const body = await createRes.text().catch(() => '')
    throw new Error(`Build creation failed (${createRes.status})${body ? `: ${body}` : ''}`)
  }

  const created = (await createRes.json()) as { build_id: string }
  const buildId = created.build_id

  const controller = new AbortController()
  const timeout = window.setTimeout(() => controller.abort(), BUILD_SSE_TIMEOUT)
  try {
    const logsRes = await fetch(`${API_BASE}/builds/${buildId}/logs`, {
      credentials: 'include',
      signal: controller.signal
    })

    if (logsRes.ok && logsRes.body) {
      const reader = logsRes.body.getReader()
      const decoder = new TextDecoder()
      let buffer = ''
      let done = false
      while (!done) {
        const chunk = await reader.read()
        if (chunk.done) break
        buffer += decoder.decode(chunk.value, { stream: true })

        for (;;) {
          const eventEnd = buffer.indexOf('\n\n')
          if (eventEnd < 0) break
          const event = buffer.slice(0, eventEnd)
          buffer = buffer.slice(eventEnd + 2)

          for (const line of event.split('\n')) {
            if (!line.startsWith('data: ')) continue
            const data = line.slice(6).trim()
            if (!data) continue
            onBuildLog(data)
          }

          if (event.includes('event: done')) {
            done = true
            break
          }
        }
      }
    }
  } catch {
    // ignore SSE timeout and fetch final build result
  } finally {
    window.clearTimeout(timeout)
  }

  const buildRes = await fetch(`${API_BASE}/builds/${buildId}`, {
    credentials: 'include'
  })
  if (!buildRes.ok) {
    throw new Error(`Failed to fetch build result (${buildRes.status})`)
  }

  const build = (await buildRes.json()) as BuildState
  if (build.status === 'failed') {
    throw new Error(build.error ?? 'Build failed')
  }
  if (build.status !== 'done' || !build.artifact?.bundle) {
    throw new Error(`Build not ready: ${build.status}`)
  }

  const completedBuild: CompletedBuild = {
    ...build,
    status: 'done',
    artifact: build.artifact
  }

  return {
    build: completedBuild,
    uploadedFiles: fileNames
  }
}
