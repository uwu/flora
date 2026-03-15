import type { InteractionContext, MessageContext } from './types'

export type Command = {
  name: string
  description?: string
  run: (ctx: MessageContext & { args: string[] }) => Promise<void> | void
}

export function prefix(command: Command): Command {
  return command
}

export type SlashCommandOption = {
  name: string
  description: string
  type?: 'string' | 'integer' | 'number' | 'boolean' | 'subcommand' | 'subcommand_group'
  required?: boolean
  options?: SlashCommandOption[]
}

export type SlashSubcommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
  run: (ctx: InteractionContext) => Promise<void> | void
}

export type SlashCommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
  subcommands?: SlashSubcommand[]
  run?: (ctx: InteractionContext) => Promise<void> | void
}

export function slash(command: SlashCommand): SlashCommand {
  return command
}

export type CreateOptions = {
  prefix?: string
  commands?: Command[]
  prefixCommands?: Command[]
  slashCommands?: SlashCommand[]
}

export type FlattenedSlashCommand = {
  name: string
  description: string
  options?: SlashCommandOption[]
}

export type SubcommandMap = Record<
  string,
  Record<string, (ctx: InteractionContext) => Promise<void> | void>
>

type CreateBotState = {
  initialized: boolean
}

declare global {
  // Internal sdk marker to keep createBot idempotent.
  var __floraCreateBotState: CreateBotState | undefined
}

function getCreateBotState(): CreateBotState {
  const state = globalThis.__floraCreateBotState
  if (state) return state

  const initialState = { initialized: false }
  globalThis.__floraCreateBotState = initialState
  return initialState
}

export function createBot(options: CreateOptions) {
  const state = getCreateBotState()
  if (state.initialized) {
    console.log('[flora/sdk] createBot called multiple times; skipping duplicate registration')
    return
  }
  state.initialized = true

  const prefix = options.prefix ?? '!'
  const commands = options.commands ?? options.prefixCommands ?? []
  const slashCommands = options.slashCommands ?? []

  on('messageCreate', async (ctx: MessageContext) => {
    if (!ctx.msg || !ctx.msg.content) return
    if (ctx.msg.author?.bot) return

    const content = ctx.msg.content.trim()
    if (!content.startsWith(prefix)) return

    const body = content.slice(prefix.length).trim()
    const [commandName, ...args] = body.split(/\s+/)
    const command = commands.find((cmd) => cmd.name === commandName)
    if (!command) return

    await command.run({ ...ctx, args })
  })

  on('interactionCreate', async (ctx: InteractionContext) => {
    if (!ctx.msg) return
    const command = slashCommands.find((cmd) => cmd.name === ctx.msg.commandName)
    if (!command) return

    if (command.subcommands && command.subcommands.length > 0) {
      await handleSubcommand(ctx, command)
    } else if (command.run) {
      const rawData = ctx.msg.data as any
      const options = flattenInteractionOptions(rawData?.options || [])
      await command.run({ ...ctx, options })
    }
  })

  if (slashCommands.length && typeof registerSlashCommands === 'function') {
    const flattenedCommands = flattenCommands(slashCommands)
    registerSlashCommands(flattenedCommands)
  }
}

export function flattenCommands(commands: SlashCommand[]): FlattenedSlashCommand[] {
  const subcommands = (globalThis as any).__floraSubcommands as SubcommandMap | undefined
  ;(globalThis as any).__floraSubcommands = subcommands || {}
  return commands.map((cmd) => {
    if (cmd.subcommands && cmd.subcommands.length > 0) {
      const submap: Record<string, (ctx: InteractionContext) => Promise<void> | void> = {}
      cmd.subcommands.forEach((sub) => {
        submap[sub.name] = sub.run
      })
      ;(globalThis as any).__floraSubcommands[cmd.name] = submap

      return {
        name: cmd.name,
        description: cmd.description,
        options: cmd.subcommands.map((sub) => ({
          name: sub.name,
          description: sub.description,
          type: 'subcommand' as const,
          options: sub.options
        }))
      }
    }

    return {
      name: cmd.name,
      description: cmd.description,
      options: cmd.options
    }
  })
}

export async function handleSubcommand(ctx: InteractionContext, command: SlashCommand) {
  const rawData = ctx.msg.data as any
  if (!rawData?.options || !Array.isArray(rawData.options)) {
    return
  }

  const firstOption = rawData.options[0]
  if (!firstOption) return

  const subcommandName = firstOption.name
  const subcommandMap = ((globalThis as any).__floraSubcommands as SubcommandMap | undefined)?.[
    command.name
  ]
  if (!subcommandMap) return

  const subcommandHandler = subcommandMap[subcommandName]
  if (!subcommandHandler) return

  const subcommandOptions = firstOption.options || []
  const flatOptions = flattenInteractionOptions(subcommandOptions)

  const enrichedCtx = {
    ...ctx,
    options: flatOptions
  }

  await subcommandHandler(enrichedCtx)
}

export function flattenInteractionOptions(options: any[]): Record<string, any> {
  const result: Record<string, any> = {}

  for (const opt of options) {
    if (opt.type === 1 || opt.type === 2) {
      Object.assign(result, flattenInteractionOptions(opt.options || []))
    } else {
      result[opt.name] = opt.value
    }
  }

  return result
}
