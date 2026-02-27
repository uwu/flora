import { loadProjectConfig } from '../lib/config'
import { bundleDeploymentSource } from '../lib/deploy_bundle'
import { authHeaders, createApiClient, expectOk } from '../lib/http'
import { logger } from '../lib/logger'
import { promptIfMissing } from '../lib/prompts'
import type { CliConfig, DeploySourceMapMode } from '../lib/types'

export type DeployOverrides = {
  root?: string
  sourcemap?: DeploySourceMapMode
  minify?: boolean
  external?: string[]
}

export async function deploy(
  config: CliConfig,
  guildArg: string | undefined,
  entryArg: string | undefined,
  overrides: DeployOverrides = {}
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const projectConfig = await loadProjectConfig()

  const bundled = await bundleDeploymentSource({
    root: overrides.root ?? projectConfig.root,
    entry: entryArg ?? projectConfig.entry,
    sourcemap: overrides.sourcemap ?? projectConfig.sourcemap,
    minify: overrides.minify ?? projectConfig.minify,
    external: overrides.external ?? projectConfig.external
  })

  const client = createApiClient(config)
  const response = await expectOk(
    client.POST('/deployments/{guild_id}', {
      params: { path: { guild_id: guild } },
      headers: authHeaders(config),
      body: {
        entry: bundled.entry,
        bundle: bundled.bundle,
        source_map: bundled.sourceMap
      }
    })
  )

  logger.log(
    `Deployed guild ${response.guild_id} entry=${response.entry} updated=${response.updated_at}`
  )
}

export async function get(config: CliConfig, guildArg: string | undefined): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')

  const client = createApiClient(config)
  const deployment = await expectOk(
    client.GET('/deployments/{guild_id}', {
      params: { path: { guild_id: guild } },
      headers: authHeaders(config)
    })
  )

  logger.log(
    `Guild ${deployment.guild_id}\n  entry: ${deployment.entry}\n  created: ${deployment.created_at}\n  updated: ${deployment.updated_at}`
  )
}

export async function list(config: CliConfig): Promise<void> {
  const client = createApiClient(config)
  const deployments = await expectOk(
    client.GET('/deployments/', {
      headers: authHeaders(config)
    })
  )

  if (deployments.length === 0) {
    logger.log('No deployments found')
    return
  }

  for (const deployment of deployments) {
    logger.log(
      `${deployment.guild_id} entry=${deployment.entry} created=${deployment.created_at} updated=${deployment.updated_at}`
    )
  }
}

export async function health(config: CliConfig): Promise<void> {
  const client = createApiClient(config)
  const response = await expectOk(
    client.GET('/health/', {
      headers: authHeaders(config),
      parseAs: 'text'
    })
  )

  logger.log(`${response}`)
}
