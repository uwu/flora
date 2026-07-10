import type { JsonValue, RawCreateDm } from '../generated'
import { flattenCommands, type FlattenedSlashCommand, type SlashCommand } from './commands'

declare const Deno: {
  core: {
    ops: {
      op_create_dm(args: RawCreateDm): Promise<JsonValue>
      op_upsert_global_commands(args: { commands: FlattenedSlashCommand[] }): Promise<void>
    }
  }
}

function ensureCustomBotRuntime(): string {
  if (globalThis.__floraRuntimeKind !== 'custom_bot' || !globalThis.__floraBotId) {
    throw new Error('customBot is only available in a custom bot runtime')
  }
  return globalThis.__floraBotId
}

/** Custom-bot-only capabilities for DMs, global commands, and runtime identity. */
export const customBot = {
  get id(): string {
    return ensureCustomBotRuntime()
  },
  createDm(userId: string): Promise<JsonValue> {
    ensureCustomBotRuntime()
    return Deno.core.ops.op_create_dm({ userId })
  },
  upsertGlobalCommands(commands: SlashCommand[]): Promise<void> {
    ensureCustomBotRuntime()
    return Deno.core.ops.op_upsert_global_commands({ commands: flattenCommands(commands) })
  }
}
