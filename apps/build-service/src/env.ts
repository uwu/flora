import path from 'node:path'

export function requireEnv(key: string): string {
  const value = process.env[key]
  if (!value) {
    throw new Error(`Missing required environment variable: ${key}`)
  }
  return value
}

export function getPort(): number {
  const raw = process.env.BUILD_SERVICE_PORT ?? process.env.PORT ?? '3001'
  const port = Number.parseInt(raw, 10)
  if (Number.isNaN(port)) {
    throw new Error(`Invalid port: ${raw}`)
  }
  return port
}

export function getHost(): string {
  return process.env.BUILD_SERVICE_HOST ?? '0.0.0.0'
}

export function getBuildWorkspaceDir(): string {
  const configured = process.env.BUILD_SERVICE_WORKSPACE_DIR
  if (configured) return path.resolve(configured)

  return path.resolve(process.cwd(), 'build-workspace')
}

export function getBuildRunsDir(): string {
  return path.join(getBuildWorkspaceDir(), 'runs')
}

export function getPnpmStoreDir(): string {
  const configured = process.env.BUILD_SERVICE_PNPM_STORE_DIR
  if (configured) return path.resolve(configured)

  return path.join(getBuildWorkspaceDir(), 'pnpm-store')
}

export function getBuildMinIntervalMs(): number {
  return getPositiveInt('BUILD_SERVICE_MIN_INTERVAL_MS', 5_000)
}

export function getMaxActiveBuildsPerGuild(): number {
  return getPositiveInt('BUILD_SERVICE_MAX_ACTIVE_PER_GUILD', 2)
}

export function getMaxActiveBuildsGlobal(): number {
  return getPositiveInt('BUILD_SERVICE_MAX_ACTIVE_GLOBAL', 8)
}

export function getMaxRetainedBuilds(): number {
  return getPositiveInt('BUILD_SERVICE_MAX_RETAINED_BUILDS', 500)
}

export function getBuildRetentionMs(): number {
  return getPositiveInt('BUILD_SERVICE_RETENTION_MS', 60 * 60 * 1000)
}

export function getMaxBuildLogLines(): number {
  return getPositiveInt('BUILD_SERVICE_MAX_LOG_LINES', 2_000)
}

export function isBundleMinifyEnabled(): boolean {
  return !isTruthy(process.env.BUILD_SERVICE_DISABLE_MINIFY)
}

function isTruthy(value: string | undefined): boolean {
  if (!value) return false
  return ['1', 'true', 'yes', 'on'].includes(value.trim().toLowerCase())
}

function getPositiveInt(key: string, fallback: number): number {
  const raw = process.env[key]
  if (!raw) return fallback

  const value = Number.parseInt(raw, 10)
  if (!Number.isFinite(value) || value < 1) {
    throw new Error(`Invalid ${key}: ${raw}`)
  }

  return value
}
