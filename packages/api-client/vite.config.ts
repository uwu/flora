import { defineConfig } from 'vite-plus'

export default defineConfig({
  pack: {
    dts: {
      tsgo: true
    },
    entry: ['src/index.ts'],
    outDir: 'dist',
    format: 'esm',
    platform: 'neutral',
    target: 'es2022',
    shims: false,
    minify: false
  }
})
