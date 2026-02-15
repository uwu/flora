import { loadConfig, saveConfig } from '../lib/config'
import { logger } from '../lib/logger'
import { promptIfMissing } from '../lib/prompts'

export async function login(tokenArg?: string): Promise<void> {
  const token = await promptIfMissing(tokenArg, 'API token')
  const config = loadConfig()
  saveConfig({
    ...config,
    token
  })

  logger.log('Saved token to config')
}
