const API_BASE = import.meta.env.VITE_API_BASE ?? '/api'

type ApiError = {
  title?: string
  detail?: string
  request_id?: string
}

export async function requestJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, { credentials: 'include', ...init })
  if (!response.ok) {
    let message = `Request failed (${response.status})`
    try {
      const body = (await response.json()) as ApiError
      if (body.detail) message = body.detail
      else if (body.title) message = body.title
    } catch {
      // ignore JSON parse errors
    }
    throw new Error(message)
  }

  return (await response.json()) as T
}
