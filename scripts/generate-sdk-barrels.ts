import { readdirSync, writeFileSync } from 'fs'
import { join } from 'path'

const GENERATED_DIR = join(import.meta.dir, '../sdk/src/generated')

const files = readdirSync(GENERATED_DIR)
  .filter(f => f.endsWith('.ts') && f !== 'index.ts' && !f.endsWith('.test.ts'))
  .sort()

const exports = files.map(file => {
  const name = file.replace('.ts', '')
  return `export * from './${name}'`
})

const content = exports.join('\n') + '\n'
writeFileSync(join(GENERATED_DIR, 'index.ts'), content)

console.log(`Generated index.ts with ${files.length} exports`)
