export function buildPong(args: string[]): string {
  const suffix = args.length > 0 ? ` (${args.join(' ')})` : ''
  return `pong${suffix}`
}
