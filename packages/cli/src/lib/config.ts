import { loadConfig as loadC12 } from 'c12'
import Conf from 'conf'

import { type CliConfig, DEFAULT_API_URL, type DeployProjectConfig } from './types'

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

export async function loadProjectConfig(cwd = process.cwd()): Promise<DeployProjectConfig> {
  const { config } = await loadC12<{ deploy?: DeployProjectConfig }>({
    cwd,
    name: 'flora',
    configFile: 'flora.config',
    rcFile: false,
    packageJson: false,
    defaults: {
      deploy: {}
    }
  })

  return config.deploy ?? {}
}
