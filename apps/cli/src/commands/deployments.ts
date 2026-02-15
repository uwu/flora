import path from 'node:path'

import { collectFiles, toRelative } from '../lib/files'
import { authHeaders, createApiClient, expectOk } from '../lib/http'
import { promptIfMissing } from '../lib/prompts'
import type { CliConfig } from '../lib/types'

export async function deploy(
  config: CliConfig,
  guildArg: string | undefined,
  entryArg: string | undefined,
  rootArg?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const entryPath = await promptIfMissing(entryArg, 'Entry file path')

  const entry = path.resolve(entryPath)
  const root = rootArg ? path.resolve(rootArg) : path.dirname(entry)

  const files = await collectFiles(root)
  const entryRel = toRelative(entry, root)

  const client = createApiClient(config)
  const response = await expectOk(
    client.POST('/deployments/{guild_id}', {
      params: { path: { guild_id: guild } },
      headers: authHeaders(config),
      body: {
        entry: entryRel,
        files
      }
    })
  )

  process.stdout.write(
    `Deployed guild ${response.guild_id} entry=${response.entry} updated=${response.updated_at}\n`
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

  process.stdout.write(
    `Guild ${deployment.guild_id}\n  entry: ${deployment.entry}\n  created: ${deployment.created_at}\n  updated: ${deployment.updated_at}\n`
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
    process.stdout.write('No deployments found\n')
    return
  }

  for (const deployment of deployments) {
    process.stdout.write(
      `${deployment.guild_id} entry=${deployment.entry} created=${deployment.created_at} updated=${deployment.updated_at}\n`
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

  process.stdout.write(`${response}\n`)
}
