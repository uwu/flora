import { cancel, isCancel, text } from '@clack/prompts'

export async function promptIfMissing(value: string | undefined, message: string): Promise<string> {
  if (value) {
    return value
  }
  if (!process.stdout.isTTY) {
    throw new Error(`missing required value: ${message}`)
  }

  const result = await text({ message })
  if (isCancel(result)) {
    cancel('Canceled')
    process.exit(1)
  }

  const next = String(result).trim()
  if (!next) {
    throw new Error(`missing required value: ${message}`)
  }

  return next
}
