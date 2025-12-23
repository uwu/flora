declare global {
  type MessageContext = import('./index').MessageContext
  type MessageUpdateContext = import('./index').MessageUpdateContext
  type MessageDeleteContext = import('./index').MessageDeleteContext
  type MessageDeleteBulkContext = import('./index').MessageDeleteBulkContext
  type InteractionContext = import('./index').InteractionContext
  type SlashCommand = import('./index').SlashCommand
  type SlashCommandOption = import('./index').SlashCommandOption
  type Command = import('./index').Command

  function on(
    event: 'messageCreate',
    handler: (ctx: MessageContext) => void | Promise<void>
  ): void
  function on(
    event: 'messageUpdate',
    handler: (ctx: MessageUpdateContext) => void | Promise<void>
  ): void
  function on(
    event: 'messageDelete',
    handler: (ctx: MessageDeleteContext) => void | Promise<void>
  ): void
  function on(
    event: 'messageDeleteBulk',
    handler: (ctx: MessageDeleteBulkContext) => void | Promise<void>
  ): void
  function on(
    event: 'interactionCreate',
    handler: (ctx: InteractionContext) => void | Promise<void>
  ): void

  const createBot: typeof import('./index').createBot
  const defineCommand: typeof import('./index').defineCommand
  const defineSlashCommand: typeof import('./index').defineSlashCommand
  const registerSlashCommands: (commands: { name: string; description?: string }[]) =>
    | Promise<void>
    | void
}

export {}
