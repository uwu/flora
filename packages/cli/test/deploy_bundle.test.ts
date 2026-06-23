import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { afterEach, describe, expect, it } from 'vite-plus/test'

import { bundleDeploymentSource } from '../src/lib/deploy_bundle'

async function createProjectFixture(): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), 'flora-cli-bundle-'))

  await mkdir(path.join(root, 'src'), { recursive: true })
  await mkdir(path.join(root, 'node_modules', 'tiny-dep'), { recursive: true })

  await writeFile(
    path.join(root, 'src', 'main.ts'),
    [
      "import { localValue } from './util'",
      "import dependencyValue from 'tiny-dep'",
      'export const value = `${localValue}${dependencyValue}`'
    ].join('\n')
  )
  await writeFile(path.join(root, 'src', 'util.ts'), "export const localValue = 'local-'\n")
  await writeFile(
    path.join(root, 'node_modules', 'tiny-dep', 'package.json'),
    JSON.stringify({ name: 'tiny-dep', version: '1.0.0', type: 'module', main: 'index.js' })
  )
  await writeFile(path.join(root, 'node_modules', 'tiny-dep', 'index.js'), "export default 'dep'\n")

  return root
}

describe('bundleDeploymentSource', () => {
  const roots: string[] = []

  afterEach(async () => {
    await Promise.all(roots.map((root) => rm(root, { recursive: true, force: true })))
    roots.length = 0
  })

  it('bundles multi-file entry and node_modules by default', async () => {
    const root = await createProjectFixture()
    roots.push(root)

    const result = await bundleDeploymentSource({ cwd: root })

    expect(result.entry).toBe('src/main.ts')
    expect(result.bundle).toContain('local-')
    expect(result.bundle).toContain('dep')
    expect(result.bundle).not.toMatch(/from ['"]tiny-dep['"]/)
  })

  it('supports sourcemap none mode', async () => {
    const root = await createProjectFixture()
    roots.push(root)

    const result = await bundleDeploymentSource({ cwd: root, sourcemap: 'none' })

    expect(result.sourceMap).toBeUndefined()
    expect(result.bundle).not.toContain('sourceMappingURL=')
  })

  it('supports sourcemap inline mode', async () => {
    const root = await createProjectFixture()
    roots.push(root)

    const result = await bundleDeploymentSource({ cwd: root, sourcemap: 'inline' })

    expect(result.sourceMap).toBeUndefined()
    expect(result.bundle).toContain('sourceMappingURL=data:')
  })

  it('supports sourcemap external mode', async () => {
    const root = await createProjectFixture()
    roots.push(root)

    const result = await bundleDeploymentSource({ cwd: root, sourcemap: 'external' })

    expect(result.sourceMap).toBeDefined()
    expect(result.sourceMap?.path).toMatch(/\.map$/)
    expect(result.sourceMap?.contents).toContain('"version"')
  })

  it('keeps explicit external imports in output', async () => {
    const root = await createProjectFixture()
    roots.push(root)

    const result = await bundleDeploymentSource({
      cwd: root,
      external: ['tiny-dep']
    })

    expect(result.bundle).toMatch(/from ['"]tiny-dep['"]/)
  })
})
