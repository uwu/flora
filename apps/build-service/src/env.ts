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

export function isBundleMinifyEnabled(): boolean {
  return !isTruthy(process.env.BUILD_SERVICE_DISABLE_MINIFY)
}

function isTruthy(value: string | undefined): boolean {
  if (!value) return false
  return ['1', 'true', 'yes', 'on'].includes(value.trim().toLowerCase())
}
