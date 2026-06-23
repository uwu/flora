import { type H3Event, HTTPError, assertBodySize } from 'h3'

import { BuildLimitError, assertCanCreateBuild, createBuild } from '../lib/builds'
import { runBuildPipeline } from '../lib/pipeline'
import { MAX_COMPRESSED_SIZE } from '../lib/zip'

const MAX_CREATE_BUILD_BODY_SIZE = MAX_COMPRESSED_SIZE + 1024 * 1024

export async function handleCreateBuild(event: H3Event) {
  await assertBodySize(event, MAX_CREATE_BUILD_BODY_SIZE)

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
  if (projectZip.size > MAX_COMPRESSED_SIZE) {
    throw new HTTPError({
      status: 413,
      message: `project_zip exceeds maximum compressed size of ${MAX_COMPRESSED_SIZE} bytes`
    })
  }

  try {
    assertCanCreateBuild(guildId)
  } catch (error) {
    if (error instanceof BuildLimitError) {
      throw new HTTPError({ status: error.status, message: error.message })
    }
    throw error
  }

  const buildId = crypto.randomUUID()
  createBuild(buildId, guildId, entry)

  const zipData = new Uint8Array(await projectZip.arrayBuffer())

  // run pipeline in the background — don't await
  void runBuildPipeline(buildId, guildId, entry, zipData)

  return { build_id: buildId, status: 'queued' as const }
}
