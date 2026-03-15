import { normalizeEdit, normalizeReply } from '../runtime/normalize'
import type { AnyPayload } from '../runtime/normalize'
import type { MessageEditOptions, MessageReplyOptions } from '../sdk/types'
import {
  makeComponentInteraction,
  makeInteraction,
  makeMessage,
  makeModalSubmit,
  makeReaction,
  makeReady
} from './factories'
import { resetIdCounter } from './id'
import { KvMock } from './kv_mock'
import type {
  ComponentInteractionPartial,
  DispatchResult,
  HarnessOptions,
  InteractionOptions,
  InteractionPartial,
  MessagePartial,
  ModalSubmitPartial,
  OpCall,
  ReactionPartial,
  ReadyPartial
} from './types'

export class TestHarness {
  private guildId: string
  private secretsMap: Record<string, string>
  private opCalls: OpCall[] = []
  private mockResponses = new Map<string, unknown | ((...args: unknown[]) => unknown)>()
  private savedGlobals: Record<string, unknown> = {}

  kv = new KvMock()

  constructor(options?: HarnessOptions) {
    this.guildId = options?.guildId ?? '999000000000000000'
    this.secretsMap = { ...(options?.secrets ?? {}) }
  }

  setup(fn: () => void): this {
    this.installGlobals()
    fn()
    return this
  }

  reset(): void {
    this.opCalls = []
    this.kv.clear()
    this.mockResponses.clear()
    resetIdCounter()
    this.removeGlobals()
    this.installGlobals()
  }

  teardown(): void {
    this.opCalls = []
    this.kv.clear()
    this.mockResponses.clear()
    this.removeGlobals()
  }

  private installGlobals(): void {
    this.savedGlobals = {
      Deno: (globalThis as any).Deno,
      __floraHandlers: (globalThis as any).__floraHandlers,
      __floraGuildId: (globalThis as any).__floraGuildId,
      __floraCreateBotState: (globalThis as any).__floraCreateBotState,
      __floraSubcommands: (globalThis as any).__floraSubcommands,
      on: (globalThis as any).on,
      __floraDispatch: (globalThis as any).__floraDispatch,
      registerSlashCommands: (globalThis as any).registerSlashCommands,
      cron: (globalThis as any).cron,
      secrets: (globalThis as any).secrets,
      console: (globalThis as any).console
    }

    const self = this

    const opsProxy = new Proxy({} as Record<string, Function>, {
      get(_target, prop: string) {
        return (...args: unknown[]): unknown => {
          self.opCalls.push({ op: prop, args, timestamp: Date.now() })

          // KV ops
          if (prop === 'op_kv_get') {
            return Promise.resolve(self.kv.get(args[0] as string, args[1] as string))
          }
          if (prop === 'op_kv_get_with_metadata') {
            return Promise.resolve(self.kv.getWithMetadata(args[0] as string, args[1] as string))
          }
          if (prop === 'op_kv_set') {
            self.kv.set(args[0] as string, args[1] as string, args[2] as string, args[3] as any)
            return Promise.resolve()
          }
          if (prop === 'op_kv_update_metadata') {
            self.kv.updateMetadata(args[0] as string, args[1] as string, args[2] as any)
            return Promise.resolve()
          }
          if (prop === 'op_kv_delete') {
            self.kv.delete(args[0] as string, args[1] as string)
            return Promise.resolve()
          }
          if (prop === 'op_kv_list_keys') {
            return Promise.resolve(self.kv.listKeys(args[0] as any, args[1] as string))
          }

          // Secrets
          if (prop === 'op_secret_placeholder') return self.secretsMap[args[0] as string]

          // Logging - no-op (captured via opCalls)
          if (prop === 'op_log') return undefined

          // Registration ops - no-op
          if (prop === 'op_register_cron') return undefined
          if (prop === 'op_upsert_guild_commands') return Promise.resolve()

          // Mock responses
          if (self.mockResponses.has(prop)) {
            const mock = self.mockResponses.get(prop)
            if (typeof mock === 'function') return Promise.resolve(mock(...args))
            return Promise.resolve(mock)
          }

          // Default: resolve void for send/edit/defer/followup ops
          return Promise.resolve()
        }
      }
    })
    ;(globalThis as any).Deno = { core: { ops: opsProxy } }
    ;(globalThis as any).__floraHandlers = {}
    ;(globalThis as any).__floraGuildId = this.guildId
    ;(globalThis as any).__floraCreateBotState = undefined
    ;(globalThis as any).__floraSubcommands = undefined
    ;(globalThis as any).secrets = {
      get: (name: string) => self.secretsMap[name]
    }
    ;(globalThis as any).on = function on(event: string, handler: Function): void {
      if (!(globalThis as any).__floraHandlers[event]) {
        ;(globalThis as any).__floraHandlers[event] = []
      }
      ;(globalThis as any).__floraHandlers[event].push(handler)
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type -- proxy always returns a function
    const ops = opsProxy as { [k: string]: Function } & {
      op_send_interaction_response(o: unknown): Promise<void>
      op_send_message(o: unknown): Promise<void>
      op_edit_message(o: unknown): Promise<void>
      op_upsert_guild_commands(o: unknown): Promise<void>
      op_register_cron(o: unknown): void
      op_log(a: unknown[]): void
    }
    const core = { ops }
    ;(globalThis as any).__floraDispatch = async function __floraDispatch(
      event: string,
      payload: unknown
    ): Promise<void> {
      const handlers = (globalThis as any).__floraHandlers[event] || []
      for (const handler of handlers) {
        const context = {
          msg: payload,
          reply(message: string | MessageReplyOptions) {
            const options = normalizeReply(message, payload as AnyPayload)
            if (options['interactionId'] && options['token']) {
              return core.ops.op_send_interaction_response(options)
            }
            return core.ops.op_send_message(options)
          },
          edit(message: string | MessageEditOptions) {
            const options = normalizeEdit(message, payload as AnyPayload)
            return core.ops.op_edit_message(options)
          }
        }
        await handler(context)
      }
    }
    ;(globalThis as any).registerSlashCommands = function registerSlashCommands(
      commands: unknown[]
    ): Promise<void> | undefined {
      if (!(globalThis as any).__floraGuildId) return
      return core.ops.op_upsert_guild_commands({
        guildId: (globalThis as any).__floraGuildId,
        commands
      }) as Promise<void>
    }

    const CRON_EVENT_PREFIX = '__cron:'
    ;(globalThis as any).cron = function cron(
      name: string,
      cronExpr: string,
      handler: Function,
      options?: { skipIfRunning?: boolean }
    ): void {
      if (typeof name !== 'string' || !name.length) {
        throw new TypeError('cron name must be a non-empty string')
      }
      if (typeof cronExpr !== 'string' || !cronExpr.length) {
        throw new TypeError('cron expression must be a non-empty string')
      }
      if (typeof handler !== 'function') throw new TypeError('cron handler must be a function')

      const eventName = CRON_EVENT_PREFIX + name
      if (!(globalThis as any).__floraHandlers[eventName]) {
        ;(globalThis as any).__floraHandlers[eventName] = []
      }
      ;(globalThis as any).__floraHandlers[eventName].push(handler)
      core.ops.op_register_cron({
        name,
        expr: cronExpr,
        skipIfRunning: options?.skipIfRunning ?? false
      })
    }
    ;(globalThis as any).console = {
      log: (...args: unknown[]) => core.ops.op_log(args)
    }
  }

  private removeGlobals(): void {
    for (const [key, value] of Object.entries(this.savedGlobals)) {
      if (value === undefined) {
        delete (globalThis as any)[key]
      } else {
        ;(globalThis as any)[key] = value
      }
    }
    this.savedGlobals = {}
  }

  // --- Simulation methods ---

  async dispatch(event: string, payload: unknown): Promise<DispatchResult> {
    const startIdx = this.opCalls.length
    await (globalThis as any).__floraDispatch(event, payload)
    return this.buildResult(startIdx)
  }

  async message(partial?: MessagePartial): Promise<DispatchResult> {
    const payload = makeMessage(partial, this.guildId)
    return this.dispatch('messageCreate', payload)
  }

  async interaction(
    name: string,
    opts?: InteractionOptions,
    partial?: InteractionPartial
  ): Promise<DispatchResult> {
    const payload = makeInteraction(name, opts, partial, this.guildId)
    return this.dispatch('interactionCreate', payload)
  }

  async componentInteraction(
    customId: string,
    partial?: ComponentInteractionPartial
  ): Promise<DispatchResult> {
    const payload = makeComponentInteraction(customId, partial, this.guildId)
    return this.dispatch('componentInteraction', payload)
  }

  async modalSubmit(
    customId: string,
    fields?: Record<string, string>,
    partial?: ModalSubmitPartial
  ): Promise<DispatchResult> {
    const payload = makeModalSubmit(customId, fields, partial, this.guildId)
    return this.dispatch('modalSubmit', payload)
  }

  async reaction(partial?: ReactionPartial): Promise<DispatchResult> {
    const payload = makeReaction(partial, this.guildId)
    return this.dispatch('reactionAdd', payload)
  }

  async ready(partial?: ReadyPartial): Promise<DispatchResult> {
    const payload = makeReady(partial)
    return this.dispatch('ready', payload)
  }

  async triggerCron(name: string): Promise<DispatchResult> {
    const payload = { name, scheduledAt: new Date().toISOString() }
    return this.dispatch(`__cron:${name}`, payload)
  }

  // --- Configuration ---

  mockResponse(opName: string, value: unknown | ((...args: unknown[]) => unknown)): void {
    this.mockResponses.set(opName, value)
  }

  setSecret(name: string, value: string): void {
    this.secretsMap[name] = value
  }

  allCalls(): OpCall[] {
    return [...this.opCalls]
  }

  // --- Internal ---

  private buildResult(startIdx: number): DispatchResult {
    const calls = this.opCalls.slice(startIdx)

    const replies = calls.filter(
      (c) => c.op === 'op_send_message' || c.op === 'op_send_interaction_response'
    )
    const edits = calls.filter((c) => c.op === 'op_edit_message')
    const defers = calls.filter((c) => c.op === 'op_defer_interaction_response')
    const followups = calls.filter((c) => c.op === 'op_create_followup_message')
    const logs = calls
      .filter((c) => c.op === 'op_log')
      .map((c) => c.args[0] as unknown[])

    return {
      calls,
      replies,
      edits,
      defers,
      followups,
      logs,
      firstReply: replies[0]?.args[0]
    }
  }
}
