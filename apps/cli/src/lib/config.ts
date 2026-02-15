import Conf from 'conf'

import { type CliConfig, DEFAULT_API_URL } from './types'

const store = new Conf<CliConfig>({
  projectName: 'flora',
  configName: 'cli',
  defaults: {
    apiUrl: DEFAULT_API_URL,
    token: undefined
  }
})

export function loadConfig(): CliConfig {
  const apiUrl = store.get('apiUrl') ?? DEFAULT_API_URL
  const token = store.get('token')
  return { apiUrl, token }
}

export function saveConfig(config: CliConfig): void {
  store.set(config)
}
