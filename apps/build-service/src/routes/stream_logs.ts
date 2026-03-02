import { createEventStream, getRouterParam, type H3Event, HTTPError } from 'h3'

import { getBuild, onBuildComplete, onBuildLog } from '../lib/builds'

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

  // replay existing logs
  for (const line of build.logs) {
    stream.push({ event: 'log', data: line })
  }

  // if already complete, send done event and close
  if (build.status === 'done' || build.status === 'failed') {
    stream.push({ event: 'done', data: JSON.stringify({ status: build.status }) })
    stream.close()
    return stream.send()
  }

  // subscribe to new log lines
  const unsubLog = onBuildLog(buildId, (line) => {
    stream.push({ event: 'log', data: line })
  })

  const unsubComplete = onBuildComplete(buildId, (completedBuild) => {
    stream.push({
      event: 'done',
      data: JSON.stringify({ status: completedBuild.status })
    })
    stream.close()
  })

  stream.onClosed(() => {
    unsubLog()
    unsubComplete()
  })

  return stream.send()
}
