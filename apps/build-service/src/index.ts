import { colors } from 'consola/utils'
import { H3, HTTPError, serve } from 'h3'

import { bearerAuth } from './auth'
import { getHost, getPort, requireEnv } from './env'
import { logger } from './lib/logger'
import { handleCreateBuild } from './routes/create-build'
import { handleGetBuild } from './routes/get-build'
import { handleStreamLogs } from './routes/stream-logs'

const secret = requireEnv('BUILD_SERVICE_SECRET')
const port = getPort()
const auth = bearerAuth(secret)

const app = new H3({ debug: process.env.NODE_ENV !== 'production' })
const requestLogger = logger.withTag(colors.cyan('http'))

app.use(async (event, next) => {
  if (event.url.pathname === '/health') {
    return next()
  }

  const startedAt = Date.now()
  const method = event.req.method
  const path = `${event.url.pathname}${event.url.search}`

  try {
    const response = await next()
    const status = event.res.status ?? (response instanceof Response ? response.status : 200)
    const elapsedMs = Date.now() - startedAt
    requestLogger.info(formatRequestLog(method, path, status, elapsedMs))
    return response
  } catch (error) {
    const status = error instanceof HTTPError ? error.status : 500
    const elapsedMs = Date.now() - startedAt
    requestLogger.error(formatRequestLog(method, path, status, elapsedMs))
    throw error
  }
})

// auth middleware for all /internal/* routes
app.use('/internal', auth)

app.post('/internal/builds', handleCreateBuild)
app.get('/internal/builds/:build_id', handleGetBuild)
app.get('/internal/builds/:build_id/logs', handleStreamLogs)

app.get('/health', () => ({ status: 'ok' }))

serve(app, { port, hostname: getHost() })

function formatRequestLog(method: string, path: string, status: number, elapsedMs: number): string {
  return `${colorMethod(method)} ${colors.white(path)} -> ${colorStatus(status)} ${colors.gray(
    `(${elapsedMs}ms)`
  )}`
}

function colorMethod(method: string): string {
  switch (method) {
    case 'GET':
      return colors.cyan(method)
    case 'POST':
      return colors.green(method)
    case 'PUT':
    case 'PATCH':
      return colors.yellow(method)
    case 'DELETE':
      return colors.red(method)
    default:
      return colors.white(method)
  }
}

function colorStatus(status: number): string {
  if (status >= 500) return colors.red(status)
  if (status >= 400) return colors.yellow(status)
  if (status >= 300) return colors.magenta(status)
  return colors.green(status)
}
