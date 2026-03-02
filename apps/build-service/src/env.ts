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
