import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { normalizeUrl } from '../src/lib/http'

describe('normalizeUrl', () => {
  it('removes trailing slash for non-root path', () => {
    assert.equal(
      normalizeUrl('http://localhost:3000/api/health/'),
      'http://localhost:3000/api/health'
    )
  })

  it('normalizes duplicated kv api prefix', () => {
    assert.equal(
      normalizeUrl('http://localhost:3000/api/kv/api/kv/stores'),
      'http://localhost:3000/api/kv/stores'
    )
  })
})
