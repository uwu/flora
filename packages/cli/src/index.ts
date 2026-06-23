#!/usr/bin/env node

import process from 'node:process'

import { defineCommand, runMain } from 'citty'

import { description, name, version } from '../package.json'
import { deploy, get, health, list } from './commands/deployments'
import {
  createStore,
  deleteStore,
  deleteValue,
  getValue,
  listKeys,
  listStores,
  setValue
} from './commands/kv'
import { login } from './commands/login'
import { logs, streamLogs } from './commands/logs'
import { loadConfig } from './lib/config'
import { logger } from './lib/logger'
import type { CliConfig } from './lib/types'

function positional(args: Record<string, unknown>, index: number): string | undefined {
  const values = Array.isArray(args._) ? args._ : []
  const value = values[index]
  return typeof value === 'string' ? value : undefined
}

function resolveConfig(args: Record<string, unknown>): CliConfig {
  const config = loadConfig()
  const argApiUrl = args['api']
  const apiUrl =
    (typeof argApiUrl === 'string' ? argApiUrl : undefined) ?? process.env.FLORA_API_URL

  if (apiUrl) {
    config.apiUrl = apiUrl
  }

  return config
}

const kvCommand = defineCommand({
  meta: {
    name: 'kv',
    description: 'KV store management'
  },
  subCommands: {
    'create-store': defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        name: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        await createStore(config, args.guild, args.name)
      }
    }),
    'list-stores': defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        await listStores(config, args.guild)
      }
    }),
    'delete-store': defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        name: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        await deleteStore(config, args.guild, args.name)
      }
    }),
    set: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false },
        key: { type: 'string', required: false },
        expiration: { type: 'string', required: false },
        metadata: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        const value = positional(args, 0)
        const expiration = args.expiration ? Number(args.expiration) : undefined
        await setValue(config, args.guild, args.store, args.key, value, expiration, args.metadata)
      }
    }),
    get: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        const key = positional(args, 0)
        await getValue(config, args.guild, args.store, key)
      }
    }),
    delete: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        const key = positional(args, 0)
        await deleteValue(config, args.guild, args.store, key)
      }
    }),
    'list-keys': defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false },
        prefix: { type: 'string', required: false },
        limit: { type: 'string', required: false },
        cursor: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        await listKeys(
          config,
          args.guild,
          args.store,
          args.prefix,
          args.limit ? Number(args.limit) : undefined,
          args.cursor
        )
      }
    })
  }
})

const main = defineCommand({
  meta: {
    name,
    description,
    version
  },
  args: {
    api: {
      type: 'string',
      required: false,
      alias: 'a'
    }
  },
  subCommands: {
    deploy: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        root: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        const entry = positional(args, 0)
        await deploy(config, args.guild, entry, args.root)
      }
    }),
    get: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        await get(config, args.guild)
      }
    }),
    list: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        await list(config)
      }
    }),
    health: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        await health(config)
      }
    }),
    login: defineCommand({
      args: {
        token: { type: 'string', required: false }
      },
      async run({ args }) {
        const positionalToken = positional(args, 0)
        await login(args.token ?? positionalToken)
      }
    }),
    logs: defineCommand({
      args: {
        api: { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: true },
        follow: { type: 'boolean', required: false, alias: 'f' },
        limit: { type: 'string', required: false, alias: 'n' }
      },
      async run({ args }) {
        const config = resolveConfig(args)
        const follow = Boolean(args.follow)
        const limit = args.limit ? Number(args.limit) : 100

        if (follow) {
          await streamLogs(config, args.guild)
          return
        }

        await logs(config, args.guild, limit)
      }
    }),
    kv: kvCommand
  }
})

runMain(main).catch((error) => {
  const message = error instanceof Error ? error.message : String(error)
  logger.error(message)
  process.exit(1)
})
