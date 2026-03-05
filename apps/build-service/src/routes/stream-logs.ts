import { createEventStream, getRouterParam, type H3Event, HTTPError } from 'h3'

import { getBuild, onBuildComplete, onBuildLog } from '../lib/builds'

function isClosedStreamError(error: unknown): boolean {
  return (
    error instanceof TypeError &&
    'code' in error &&
    typeof error.code === 'string' &&
    error.code === 'ERR_INVALID_STATE'
  )
}

export function handleStreamLogs(event: H3Event) {
  const buildId = getRouterParam(event, 'build_id')
  if (!buildId) {
    throw new HTTPError({ status: 400, message: 'build_id is required' })
  }

  const build = getBuild(buildId)
  if (!build) {
    throw new HTTPError({ status: 404, message: `Build ${buildId} not found` })
  }

  const stream = createEventStream(event)
  let closed = false

  const closeStream = () => {
    if (closed) return
    closed = true
    void stream.close()
  }

  const pushSafe = async (message: { event: string; data: string }) => {
    if (closed) return false

    try {
      await stream.push(message)
      return true
    } catch (error) {
      if (isClosedStreamError(error)) {
        closeStream()
        return false
      }

      closeStream()
      return false
    }
  }

  // replay existing logs
  for (const line of build.logs) {
    void pushSafe({ event: 'log', data: line })
  }

  // if already complete, send done event and close
  if (build.status === 'done' || build.status === 'failed') {
    void pushSafe({ event: 'done', data: JSON.stringify({ status: build.status }) })
    closeStream()
    return stream.send()
  }

  // subscribe to new log lines
  const unsubLog = onBuildLog(buildId, (line) => {
    void pushSafe({ event: 'log', data: line })
  })

  const unsubComplete = onBuildComplete(buildId, (completedBuild) => {
    void pushSafe({
      event: 'done',
      data: JSON.stringify({ status: completedBuild.status })
    })
    closeStream()
  })

  stream.onClosed(() => {
    closed = true
    unsubLog()
    unsubComplete()
  })

  return stream.send()
}
