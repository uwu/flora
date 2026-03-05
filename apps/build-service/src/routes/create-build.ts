import { type H3Event, HTTPError } from 'h3'

import { createBuild } from '../lib/builds'
import { runBuildPipeline } from '../lib/pipeline'

export async function handleCreateBuild(event: H3Event) {
  const form = await event.req.formData()

  const guildId = form.get('guild_id')
  const entry = form.get('entry')
  const projectZip = form.get('project_zip')

  if (!guildId || typeof guildId !== 'string') {
    throw new HTTPError({ status: 400, message: 'guild_id is required' })
  }
  if (!entry || typeof entry !== 'string') {
    throw new HTTPError({ status: 400, message: 'entry is required' })
  }
  if (!(projectZip instanceof File)) {
    throw new HTTPError({ status: 400, message: 'project_zip must be a file upload' })
  }

  const buildId = crypto.randomUUID()
  createBuild(buildId, guildId, entry)

  const zipData = new Uint8Array(await projectZip.arrayBuffer())

  // run pipeline in the background — don't await
  runBuildPipeline(buildId, guildId, entry, zipData)

  return { build_id: buildId, status: 'queued' as const }
}
