import fs from 'node:fs/promises'
import path from 'node:path'

const MAX_DEPENDENCY_COUNT = 50

const DISALLOWED_SPECIFIER_PREFIXES = [
  'file:',
  'link:',
  'workspace:',
  'catalog:',
  'npm:',
  'patch:',
  'portal:',
  'git+ssh:',
  'git+https:',
  'git:',
  'github:',
  'bitbucket:',
  'gitlab:',
  'http://',
  'https://'
]

const DISALLOWED_PACKAGE_MANAGER_FILES = [
  '.npmrc',
  '.pnpmfile.cjs',
  '.pnpmfile.mjs',
  '.yarnrc',
  '.yarnrc.yml',
  'bun.lock',
  'bun.lockb',
  'npm-shrinkwrap.json',
  'package-lock.json',
  'pnpm-lock.yaml',
  'pnpm-workspace.yaml',
  'pnpm-workspace.yml',
  'yarn.lock'
]

// Matches bare GitHub shorthands like "user/repo" or "user/repo#ref"
// Scoped npm packages (@org/pkg) are excluded since they start with "@"
const BARE_GIT_SHORTHAND_RE = /^[a-zA-Z0-9_.-]+\/[a-zA-Z0-9_.-]/

export type ValidatedPackageJson = {
  name?: string
  version?: string
  dependencies?: Record<string, string>
}

export async function validateAndSanitizePackageJson(
  workspaceDir: string
): Promise<ValidatedPackageJson> {
  for (const file of DISALLOWED_PACKAGE_MANAGER_FILES) {
    const hasFile = await fs
      .access(path.join(workspaceDir, file))
      .then(() => true)
      .catch(() => false)

    if (hasFile) {
      throw new Error(`Disallowed package-manager file: ${file}`)
    }
  }

  const pkgPath = path.join(workspaceDir, 'package.json')
  const raw = await fs.readFile(pkgPath, 'utf-8')
  const pkg = JSON.parse(raw) as Record<string, unknown>

  const sanitized: ValidatedPackageJson = {
    name: typeof pkg.name === 'string' ? pkg.name : undefined,
    version: typeof pkg.version === 'string' ? pkg.version : undefined
  }

  if (
    pkg.dependencies &&
    typeof pkg.dependencies === 'object' &&
    !Array.isArray(pkg.dependencies)
  ) {
    const deps = pkg.dependencies as Record<string, unknown>
    const validDeps: Record<string, string> = {}

    const entries = Object.entries(deps)

    if (entries.length > MAX_DEPENDENCY_COUNT) {
      throw new Error(
        `Too many dependencies: ${entries.length} exceeds limit of ${MAX_DEPENDENCY_COUNT}`
      )
    }

    for (const [name, specifier] of entries) {
      if (typeof specifier !== 'string') {
        throw new Error(`Invalid specifier for dependency "${name}": must be a string`)
      }

      for (const prefix of DISALLOWED_SPECIFIER_PREFIXES) {
        if (specifier.startsWith(prefix)) {
          throw new Error(
            `Disallowed specifier for dependency "${name}": "${specifier}" (${prefix} not allowed)`
          )
        }
      }

      if (specifier.includes('://')) {
        throw new Error(`Disallowed URL-like specifier for dependency "${name}": "${specifier}"`)
      }

      if (specifier.startsWith('/') || specifier.startsWith('./') || specifier.startsWith('../')) {
        throw new Error(`Disallowed local path specifier for dependency "${name}": "${specifier}"`)
      }

      // Block bare GitHub shorthands like "user/repo" or "user/repo#ref"
      // Scoped packages like "@org/pkg" are safe — they start with "@"
      if (!specifier.startsWith('@') && BARE_GIT_SHORTHAND_RE.test(specifier)) {
        throw new Error(`Disallowed bare Git shorthand for dependency "${name}": "${specifier}"`)
      }

      validDeps[name] = specifier
    }

    sanitized.dependencies = validDeps
  }

  // Write sanitized package.json back (strips scripts, devDependencies, etc.)
  await fs.writeFile(pkgPath, JSON.stringify(sanitized, null, 2) + '\n')

  return sanitized
}
