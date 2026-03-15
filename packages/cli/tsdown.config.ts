import { defineConfig } from 'vite-plus/pack'

export default defineConfig({
  dts: {
    tsgo: true
  },
  entry: ['src/index.ts'],
  outDir: 'dist',
  format: 'esm',
  platform: 'node',
  target: 'node20',
  clean: true,
  shims: false,
  minify: false
})
