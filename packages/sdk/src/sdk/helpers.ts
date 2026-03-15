import type { InteractionContext } from './types'

export function hasRole(ctx: InteractionContext, roleId: string): boolean {
  return ctx.msg.member?.roles?.includes(roleId) ?? false
}

export function getSubcommand(ctx: InteractionContext): string | undefined {
  const rawData = ctx.msg.data as any
  if (!rawData?.options || !Array.isArray(rawData.options)) return undefined
  return rawData.options[0]?.name
}

export function getSubcommandGroup(ctx: InteractionContext): string | undefined {
  const rawData = ctx.msg.data as any
  if (!rawData?.options || !Array.isArray(rawData.options)) return undefined

  const firstOption = rawData.options[0]
  if (!firstOption) return undefined

  const type = firstOption.type
  if (type === 2) {
    return firstOption.name
  }

  return undefined
}
