import { requestJson } from './api/request'

export type ServerCustomBot = {
  guild_id: string
  bot_user_id: string
  bot_username: string
  application_id: string
  running: boolean
  created_at: string
  updated_at: string
}

export const serverCustomBotsApi = {
  get: (guildId: string) => requestJson<ServerCustomBot | null>(`/server-custom-bots/${guildId}`),
  upsert: (guildId: string, token: string) =>
    requestJson<ServerCustomBot>(`/server-custom-bots/${guildId}`, {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ token })
    }),
  delete: (guildId: string) =>
    requestJson<null>(`/server-custom-bots/${guildId}`, { method: 'DELETE' })
}
