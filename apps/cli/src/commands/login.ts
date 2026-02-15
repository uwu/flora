import { loadConfig, saveConfig } from '../lib/config'
import { promptIfMissing } from '../lib/prompts'

export async function login(tokenArg?: string): Promise<void> {
  const token = await promptIfMissing(tokenArg, 'API token')
  const config = loadConfig()
  saveConfig({
    ...config,
    token
  })

  process.stdout.write('Saved token to config\n')
}
