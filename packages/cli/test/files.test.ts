import { describe, expect, it } from 'vitest'

import { toRelative } from '../src/lib/files'

describe('toRelative', () => {
  it('normalizes separators', () => {
    const root = '/tmp/project'
    const file = '/tmp/project/src/main.ts'
    expect(toRelative(file, root)).toBe('src/main.ts')
  })

  it('throws when outside root', () => {
    expect(() => toRelative('/tmp/other/main.ts', '/tmp/project')).toThrow()
  })
})
