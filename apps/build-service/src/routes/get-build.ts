import { getRouterParam, type H3Event, HTTPError } from 'h3'

import { getBuild } from '../lib/builds'

export function handleGetBuild(event: H3Event) {
  const buildId = getRouterParam(event, 'build_id')
  if (!buildId) {
    throw new HTTPError({ status: 400, message: 'build_id is required' })
  }

  const build = getBuild(buildId)
  if (!build) {
    throw new HTTPError({ status: 404, message: `Build ${buildId} not found` })
  }

  return {
    build_id: build.build_id,
    guild_id: build.guild_id,
    entry: build.entry,
    status: build.status,
    logs: build.logs,
    started_at: build.started_at,
    finished_at: build.finished_at,
    artifact: build.artifact,
    error: build.error
  }
}
