import { describe, expect, it } from 'vitest'

import { normalizeUrl } from '../src/lib/http'

describe('normalizeUrl', () => {
  it('removes trailing slash for non-root path', () => {
    expect(
      normalizeUrl('http://localhost:3000/api/health/')
    ).toBe('http://localhost:3000/api/health')
  })

  it('normalizes duplicated kv api prefix', () => {
    expect(
      normalizeUrl('http://localhost:3000/api/kv/api/kv/stores')
    ).toBe('http://localhost:3000/api/kv/stores')
  })
})
