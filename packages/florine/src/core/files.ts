import { comparePaths, normalizePath } from '../utils/path.ts'

export type FlorineFile = {
  path: string
  content: string
}

export type FlorineFileMap = Record<string, string>

export const defaultFiles: readonly FlorineFile[] = [
  {
    path: 'README.md',
    content: '# Florine\n\nA lightweight VS Code-inspired editor shell.\n'
  },
  {
    path: 'src/index.ts',
    content: "export const greeting = 'hello from Florine'\n"
  }
]

export function normalizeFiles(
  files: readonly FlorineFile[] | FlorineFileMap
): readonly FlorineFile[] {
  const entries = Array.isArray(files)
    ? files
    : Object.entries(files).map(([path, content]) => ({ path, content }))

  const deduped = new Map<string, string>()
  for (const file of entries) {
    const path = normalizePath(file.path)
    if (path) deduped.set(path, file.content)
  }

  return [...deduped.entries()]
    .map(([path, content]) => ({ path, content }))
    .sort((left, right) => comparePaths(left.path, right.path))
}

export function renameFiles(
  files: readonly FlorineFile[],
  sourcePath: string,
  destinationPath: string
): readonly FlorineFile[] {
  const sourcePrefix = `${sourcePath}/`
  const next = files.map((file) => {
    if (file.path === sourcePath) return { ...file, path: destinationPath }
    if (file.path.startsWith(sourcePrefix)) {
      return { ...file, path: `${destinationPath}/${file.path.slice(sourcePrefix.length)}` }
    }
    return file
  })

  return normalizeFiles(next)
}
