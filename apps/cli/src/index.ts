#!/usr/bin/env node

import process from 'node:process'

import { defineCommand, runMain } from 'citty'

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
import type { CliConfig } from './lib/types'

function positional(args: Record<string, unknown>, index: number): string | undefined {
  const values = Array.isArray(args._) ? args._ : []
  const value = values[index]
  return typeof value === 'string' ? value : undefined
}

function cliApiUrlFromArgv(): string | undefined {
  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i]
    if (!arg) {
      continue
    }
    if (arg.startsWith('--api-url=')) {
      return arg.slice('--api-url='.length)
    }
    if (arg === '--api-url' || arg === '-a') {
      const next = process.argv[i + 1]
      if (next && !next.startsWith('-')) {
        return next
      }
    }
  }

  return undefined
}

function resolveConfig(args: Record<string, unknown>): CliConfig {
  const config = loadConfig()
  const apiUrl = (args['api-url'] as string | undefined) ?? cliApiUrlFromArgv() ??
    process.env.FLORA_API_URL
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
        'api-url': { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        name: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        await createStore(config, args.guild as string | undefined, args.name as string | undefined)
      }
    }),
    'list-stores': defineCommand({
      args: {
        'api-url': { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        await listStores(config, args.guild as string | undefined)
      }
    }),
    'delete-store': defineCommand({
      args: {
        'api-url': { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        name: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        await deleteStore(config, args.guild as string | undefined, args.name as string | undefined)
      }
    }),
    set: defineCommand({
      args: {
        'api-url': { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false },
        key: { type: 'string', required: false },
        expiration: { type: 'string', required: false },
        metadata: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        const value = positional(args as Record<string, unknown>, 0)
        const expiration = args.expiration ? Number(args.expiration) : undefined
        await setValue(
          config,
          args.guild as string | undefined,
          args.store as string | undefined,
          args.key as string | undefined,
          value,
          expiration,
          args.metadata as string | undefined
        )
      }
    }),
    get: defineCommand({
      args: {
        'api-url': { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        const key = positional(args as Record<string, unknown>, 0)
        await getValue(
          config,
          args.guild as string | undefined,
          args.store as string | undefined,
          key
        )
      }
    }),
    delete: defineCommand({
      args: {
        'api-url': { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        const key = positional(args as Record<string, unknown>, 0)
        await deleteValue(
          config,
          args.guild as string | undefined,
          args.store as string | undefined,
          key
        )
      }
    }),
    'list-keys': defineCommand({
      args: {
        'api-url': { type: 'string', required: false, alias: 'a' },
        guild: { type: 'string', required: false },
        store: { type: 'string', required: false },
        prefix: { type: 'string', required: false },
        limit: { type: 'string', required: false },
        cursor: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        await listKeys(
          config,
          args.guild as string | undefined,
          args.store as string | undefined,
          args.prefix as string | undefined,
          args.limit ? Number(args.limit) : undefined,
          args.cursor as string | undefined
        )
      }
    })
  }
})

const main = defineCommand({
  meta: {
    name: 'flora',
    description: 'Deployment CLI for flora guild scripts',
    version: '0.0.0'
  },
  args: {
    'api-url': {
      type: 'string',
      required: false,
      alias: 'a'
    }
  },
  subCommands: {
    deploy: defineCommand({
      args: {
        guild: { type: 'string', required: false },
        root: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        const entry = positional(args as Record<string, unknown>, 0)
        await deploy(
          config,
          args.guild as string | undefined,
          entry,
          args.root as string | undefined
        )
      }
    }),
    get: defineCommand({
      args: {
        guild: { type: 'string', required: false }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        await get(config, args.guild as string | undefined)
      }
    }),
    list: defineCommand({
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        await list(config)
      }
    }),
    health: defineCommand({
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        await health(config)
      }
    }),
    login: defineCommand({
      args: {
        token: { type: 'string', required: false }
      },
      async run({ args }) {
        const positionalToken = positional(args as Record<string, unknown>, 0)
        await login((args.token as string | undefined) ?? positionalToken)
      }
    }),
    logs: defineCommand({
      args: {
        guild: { type: 'string', required: false },
        follow: { type: 'boolean', required: false, alias: 'f' },
        limit: { type: 'string', required: false, alias: 'n' }
      },
      async run({ args }) {
        const config = resolveConfig(args as Record<string, unknown>)
        const follow = Boolean(args.follow)
        const limit = args.limit ? Number(args.limit) : 100

        if (follow) {
          await streamLogs(config, args.guild as string | undefined)
          return
        }

        await logs(config, args.guild as string | undefined, limit)
      }
    }),
    kv: kvCommand
  }
})

runMain(main).catch((error) => {
  const message = error instanceof Error ? error.message : String(error)
  process.stderr.write(`${message}\n`)
  process.exit(1)
})
