import { defineConfig } from 'tsdown'

export default defineConfig({
  entry: ['src/index.ts'],
  outDir: 'dist',
  format: 'esm',
  platform: 'node',
  target: 'node20',
  clean: true,
  shims: false,
  minify: false
})
