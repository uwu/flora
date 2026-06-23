import { EventEmitter } from 'node:events'

import {
  getBuildMinIntervalMs,
  getBuildRetentionMs,
  getMaxActiveBuildsGlobal,
  getMaxActiveBuildsPerGuild,
  getMaxBuildLogLines,
  getMaxRetainedBuilds
} from '../env'

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
const lastBuildCreatedAtByGuild = new Map<string, number>()

const MAX_ACTIVE_BUILDS_PER_GUILD = getMaxActiveBuildsPerGuild()
const MAX_ACTIVE_BUILDS_GLOBAL = getMaxActiveBuildsGlobal()
const MIN_BUILD_INTERVAL_MS = getBuildMinIntervalMs()
const MAX_RETAINED_BUILDS = getMaxRetainedBuilds()
const BUILD_RETENTION_MS = getBuildRetentionMs()
const MAX_LOG_LINES = getMaxBuildLogLines()

export class BuildLimitError extends Error {
  constructor(
    message: string,
    public readonly status: number
  ) {
    super(message)
  }
}

export function assertCanCreateBuild(guildId: string): void {
  pruneBuilds()

  const now = Date.now()
  const lastCreatedAt = lastBuildCreatedAtByGuild.get(guildId)
  if (lastCreatedAt && now - lastCreatedAt < MIN_BUILD_INTERVAL_MS) {
    throw new BuildLimitError('Build creation is rate limited for this guild', 429)
  }

  let activeGlobal = 0
  let activeForGuild = 0
  for (const build of builds.values()) {
    if (!isActiveStatus(build.status)) continue

    activeGlobal += 1
    if (build.guild_id === guildId) activeForGuild += 1
  }

  if (activeForGuild >= MAX_ACTIVE_BUILDS_PER_GUILD) {
    throw new BuildLimitError('Too many active builds for this guild', 429)
  }

  if (activeGlobal >= MAX_ACTIVE_BUILDS_GLOBAL) {
    throw new BuildLimitError('Too many active builds', 503)
  }
}

export function createBuild(buildId: string, guildId: string, entry: string): Build {
  pruneBuilds()

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
  lastBuildCreatedAtByGuild.set(guildId, Date.now())
  return build
}

export function getBuild(buildId: string): Build | undefined {
  return builds.get(buildId)
}

export function appendLog(buildId: string, line: string): void {
  const build = builds.get(buildId)
  if (!build) return
  build.logs.push(line)
  if (build.logs.length > MAX_LOG_LINES) {
    build.logs.splice(0, build.logs.length - MAX_LOG_LINES)
  }
  buildEvents.emit(`log:${buildId}`, line)
}

export function updateBuildStatus(buildId: string, status: BuildStatus): void {
  const build = builds.get(buildId)
  if (!build) return
  if (isTerminalStatus(build.status) && build.status !== status) return

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
  if (isTerminalStatus(build.status)) return
  build.artifact = artifact
}

export function setBuildError(buildId: string, error: string): void {
  const build = builds.get(buildId)
  if (!build) return
  if (build.status === 'done') return
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

function isActiveStatus(status: BuildStatus): boolean {
  return status === 'queued' || status === 'running'
}

function isTerminalStatus(status: BuildStatus): boolean {
  return status === 'done' || status === 'failed'
}

function pruneBuilds(): void {
  const now = Date.now()

  for (const [buildId, build] of builds) {
    if (isActiveStatus(build.status)) continue

    const finishedAt = build.finished_at ? Date.parse(build.finished_at) : Number.NaN
    if (Number.isFinite(finishedAt) && now - finishedAt > BUILD_RETENTION_MS) {
      builds.delete(buildId)
    }
  }

  while (builds.size > MAX_RETAINED_BUILDS) {
    const oldest = builds.entries().next().value as [string, Build] | undefined
    if (!oldest) break

    if (isActiveStatus(oldest[1].status)) break
    builds.delete(oldest[0])
  }
}
