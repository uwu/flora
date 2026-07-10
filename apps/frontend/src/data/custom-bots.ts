import { requestJson } from './api/request'

export type CustomBot = {
  id: string
  label?: string | null
  bot_user_id: string
  bot_username: string
  application_id: string
  has_deployment: boolean
  running: boolean
  created_at: string
  updated_at: string
}

export type CustomBotDeployment = {
  entry: string
  files?: Array<{ path: string; contents: string }> | null
  source_map?: { path: string; contents: string } | null
  bundle?: string | null
  created_at: string
  updated_at: string
}

export const customBotScopeId = (botId: string) => `__flora_custom_bot__:${botId}`

export const customBotsApi = {
  list: () => requestJson<CustomBot[]>('/custom-bots/'),
  create: (body: { label?: string; token: string }) =>
    requestJson<CustomBot>('/custom-bots/', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(body)
    }),
  delete: (botId: string) => requestJson<null>(`/custom-bots/${botId}`, { method: 'DELETE' }),
  deployment: (botId: string) =>
    requestJson<CustomBotDeployment>(`/custom-bots/${botId}/deployment`),
  deploy: (
    botId: string,
    body: {
      entry: string
      files: Array<{ path: string; contents: string }>
      bundle: string
      source_map?: { path: string; contents: string }
    }
  ) =>
    requestJson<CustomBotDeployment>(`/custom-bots/${botId}/deployment`, {
      method: 'POST',
      headers: { 'content-type': 'application/json', 'x-flora-deploy-source': 'webui' },
      body: JSON.stringify(body)
    })
}
