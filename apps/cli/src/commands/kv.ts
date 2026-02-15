import type { components } from '../generated/openapi-schema'
import { authHeaders, createApiClient, expectOk } from '../lib/http'
import { logger } from '../lib/logger'
import { promptIfMissing } from '../lib/prompts'
import type { CliConfig } from '../lib/types'

export async function createStore(
  config: CliConfig,
  guildArg?: string,
  nameArg?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const name = await promptIfMissing(nameArg, 'Store name')

  const client = createApiClient(config)
  const response = await expectOk<components['schemas']['CreateStoreResponse']>(
    client.POST('/kv/api/kv/stores', {
      headers: authHeaders(config),
      body: {
        guild_id: guild,
        store_name: name
      }
    })
  )

  logger.log(
    `Created KV store '${response.store.store_name}' for guild ${response.store.guild_id}`
  )
}

export async function listStores(config: CliConfig, guildArg?: string): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')

  const client = createApiClient(config)
  const stores = await expectOk<components['schemas']['KvStore'][]>(
    client.GET('/kv/api/kv/stores', {
      headers: authHeaders(config),
      params: { query: { guild_id: guild } }
    })
  )

  if (stores.length === 0) {
    logger.log(`No KV stores found for guild ${guild}`)
    return
  }

  logger.log(`KV stores for guild ${guild}:`)
  for (const store of stores) {
    logger.log(`  - ${store.store_name}`)
  }
}

export async function deleteStore(
  config: CliConfig,
  guildArg?: string,
  nameArg?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const name = await promptIfMissing(nameArg, 'Store name')

  const client = createApiClient(config)
  await expectOk(
    client.DELETE('/kv/api/kv/stores/{guild_id}/{store_name}', {
      headers: authHeaders(config),
      params: {
        path: {
          guild_id: guild,
          store_name: name
        }
      }
    })
  )

  logger.log(`Deleted KV store '${name}' for guild ${guild}`)
}

export async function setValue(
  config: CliConfig,
  guildArg: string | undefined,
  storeArg: string | undefined,
  keyArg: string | undefined,
  valueArg: string | undefined,
  expiration?: number,
  metadata?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const store = await promptIfMissing(storeArg, 'Store name')
  const key = await promptIfMissing(keyArg, 'Key')
  const value = await promptIfMissing(valueArg, 'Value')

  const metadataValue = metadata ? JSON.parse(metadata) : undefined

  const client = createApiClient(config)
  await expectOk(
    client.PUT('/kv/api/kv/{guild_id}/{store_name}/{key}', {
      headers: authHeaders(config),
      params: {
        path: {
          guild_id: guild,
          store_name: store,
          key
        }
      },
      body: {
        value,
        expiration,
        metadata: metadataValue
      }
    })
  )

  logger.log(`Set ${key}=${value} in store '${store}' for guild ${guild}`)
}

export async function getValue(
  config: CliConfig,
  guildArg?: string,
  storeArg?: string,
  keyArg?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const store = await promptIfMissing(storeArg, 'Store name')
  const key = await promptIfMissing(keyArg, 'Key')

  const client = createApiClient(config)
  const response = await expectOk<components['schemas']['GetValueResponse']>(
    client.GET('/kv/api/kv/{guild_id}/{store_name}/{key}', {
      headers: authHeaders(config),
      params: {
        path: {
          guild_id: guild,
          store_name: store,
          key
        }
      }
    })
  )

  if (response.value == null) {
    logger.log(`Key '${key}' not found`)
    return
  }

  logger.log(`${response.value}`)
}

export async function deleteValue(
  config: CliConfig,
  guildArg?: string,
  storeArg?: string,
  keyArg?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const store = await promptIfMissing(storeArg, 'Store name')
  const key = await promptIfMissing(keyArg, 'Key')

  const client = createApiClient(config)
  await expectOk(
    client.DELETE('/kv/api/kv/{guild_id}/{store_name}/{key}', {
      headers: authHeaders(config),
      params: {
        path: {
          guild_id: guild,
          store_name: store,
          key
        }
      }
    })
  )

  logger.log(`Deleted key '${key}' from store '${store}' for guild ${guild}`)
}

export async function listKeys(
  config: CliConfig,
  guildArg?: string,
  storeArg?: string,
  prefix?: string,
  limit?: number,
  cursor?: string
): Promise<void> {
  const guild = await promptIfMissing(guildArg, 'Guild ID')
  const store = await promptIfMissing(storeArg, 'Store name')

  const client = createApiClient(config)
  const response = await expectOk(
    client.GET('/kv/api/kv/{guild_id}/{store_name}', {
      headers: authHeaders(config),
      params: {
        path: {
          guild_id: guild,
          store_name: store
        },
        query: {
          prefix,
          limit,
          cursor
        }
      }
    })
  )

  if (response.keys.length === 0) {
    logger.log(`No keys found in store '${store}'`)
    return
  }

  logger.log(`Keys in store '${store}' (${response.keys.length} shown):`)
  for (const key of response.keys) {
    const expires = key.expiration ? ` (expires: ${key.expiration})` : ''
    const meta = key.metadata ? ` [metadata: ${JSON.stringify(key.metadata)}]` : ''
    logger.log(`  - ${key.name}${expires}${meta}`)
  }

  const listComplete = 'list_complete' in response ? response.list_complete : response.listComplete
  if (!listComplete && response.cursor) {
    logger.log(`More keys available. Use --cursor ${response.cursor}`)
  }
}
