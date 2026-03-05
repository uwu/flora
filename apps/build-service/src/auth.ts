import { HTTPError, type Middleware } from 'h3'
import { timingSafeEqual } from 'node:crypto'

function constantTimeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false
  const bufA = Buffer.from(a)
  const bufB = Buffer.from(b)
  return timingSafeEqual(bufA, bufB)
}

export function bearerAuth(secret: string): Middleware {
  return async (event, next) => {
    const header = event.req.headers.get('authorization') ?? ''
    const parts = header.split(' ')

    if (parts.length !== 2 || parts[0]!.toLowerCase() !== 'bearer' || !parts[1]) {
      throw new HTTPError({
        status: 401,
        message: 'Missing or malformed Authorization header',
        headers: { 'www-authenticate': 'Bearer realm="internal"' }
      })
    }

    if (!constantTimeEqual(parts[1], secret)) {
      throw new HTTPError({ status: 403, message: 'Invalid token' })
    }

    return next()
  }
}
