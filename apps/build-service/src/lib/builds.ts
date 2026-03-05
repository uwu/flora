import { EventEmitter } from 'node:events'

export type BuildStatus = 'queued' | 'running' | 'done' | 'failed'

export type BuildArtifact = {
  bundle: string
  sourceMap: string
}

export type Build = {
  build_id: string
  guild_id: string
  entry: string
  status: BuildStatus
  logs: string[]
  started_at: string | null
  finished_at: string | null
  artifact: BuildArtifact | null
  error: string | null
}

export type BuildResult = {
  build_id: string
  status: BuildStatus
}

const builds = new Map<string, Build>()
const buildEvents = new EventEmitter()
buildEvents.setMaxListeners(100)

export function createBuild(buildId: string, guildId: string, entry: string): Build {
  const build: Build = {
    build_id: buildId,
    guild_id: guildId,
    entry,
    status: 'queued',
    logs: [],
    started_at: null,
    finished_at: null,
    artifact: null,
    error: null
  }
  builds.set(buildId, build)
  return build
}

export function getBuild(buildId: string): Build | undefined {
  return builds.get(buildId)
}

export function appendLog(buildId: string, line: string): void {
  const build = builds.get(buildId)
  if (!build) return
  build.logs.push(line)
  buildEvents.emit(`log:${buildId}`, line)
}

export function updateBuildStatus(buildId: string, status: BuildStatus): void {
  const build = builds.get(buildId)
  if (!build) return
  build.status = status
  if (status === 'running' && !build.started_at) {
    build.started_at = new Date().toISOString()
  }
  if (status === 'done' || status === 'failed') {
    build.finished_at = new Date().toISOString()
    buildEvents.emit(`complete:${buildId}`, build)
  }
}

export function setBuildArtifact(buildId: string, artifact: BuildArtifact): void {
  const build = builds.get(buildId)
  if (!build) return
  build.artifact = artifact
}

export function setBuildError(buildId: string, error: string): void {
  const build = builds.get(buildId)
  if (!build) return
  build.error = error
}

export function onBuildLog(buildId: string, listener: (line: string) => void): () => void {
  const event = `log:${buildId}`
  buildEvents.on(event, listener)
  return () => buildEvents.off(event, listener)
}

export function onBuildComplete(buildId: string, listener: (build: Build) => void): () => void {
  const event = `complete:${buildId}`
  buildEvents.on(event, listener)
  return () => buildEvents.off(event, listener)
}
