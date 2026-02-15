import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { toRelative } from '../src/lib/files'

describe('toRelative', () => {
  it('normalizes separators', () => {
    const root = '/tmp/project'
    const file = '/tmp/project/src/main.ts'
    assert.equal(toRelative(file, root), 'src/main.ts')
  })

  it('throws when outside root', () => {
    assert.throws(() => toRelative('/tmp/other/main.ts', '/tmp/project'))
  })
})
