export function normalizePath(path: string) {
  return path.trim().replace(/^\.\//, '').replace(/^\//, '').replace(/\/$/, '')
}

export function basename(path: string) {
  return path.split('/').at(-1) || path
}

export function comparePaths(left: string, right: string) {
  return left.localeCompare(right, undefined, { sensitivity: 'base' })
}

export function renameOpenPath(
  path: string | null,
  sourcePath: string,
  destinationPath: string
): string | null {
  if (!path) return null
  if (path === sourcePath) return destinationPath

  const sourcePrefix = `${sourcePath}/`
  if (path.startsWith(sourcePrefix)) return `${destinationPath}/${path.slice(sourcePrefix.length)}`

  return path
}

export function languageForPath(path: string) {
  const extension = path.split('.').at(-1)

  switch (extension) {
    case 'css':
      return 'css'
    case 'html':
      return 'html'
    case 'json':
      return 'json'
    case 'md':
    case 'mdx':
      return 'markdown'
    case 'ts':
    case 'tsx':
      return 'typescript'
    case 'js':
    case 'jsx':
      return 'javascript'
    default:
      return 'plaintext'
  }
}

export function joinClasses(...classes: readonly (string | undefined)[]) {
  return classes.filter(isString).join(' ')
}

export function isString(value: string | null | undefined): value is string {
  return typeof value === 'string' && value.length > 0
}
