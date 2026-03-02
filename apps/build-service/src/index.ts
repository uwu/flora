import consola from 'consola'
import { H3, serve } from 'h3'

import { bearerAuth } from './auth'
import { getPort, requireEnv } from './env'
import { handleCreateBuild } from './routes/create_build'
import { handleGetBuild } from './routes/get_build'
import { handleStreamLogs } from './routes/stream_logs'

const secret = requireEnv('BUILD_SERVICE_SECRET')
const port = getPort()
const auth = bearerAuth(secret)

const app = new H3({ debug: process.env.NODE_ENV !== 'production' })

// auth middleware for all /internal/* routes
app.use('/internal', auth)

app.post('/internal/builds', handleCreateBuild)
app.get('/internal/builds/:build_id', handleGetBuild)
app.get('/internal/builds/:build_id/logs', handleStreamLogs)

// health check (no auth)
app.get('/health', () => ({ status: 'ok' }))

serve(app, { port })
consola.info(`Build service listening on port ${port}`)
