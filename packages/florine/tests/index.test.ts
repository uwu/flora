import { expect, test } from 'vite-plus/test'
import { Florine } from '../src/index.ts'

test('exports the Florine component', () => {
  expect(typeof Florine).toBe('function')
})
