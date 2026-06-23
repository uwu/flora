import { defineConfig } from 'vite-plus/test/config'

export default defineConfig({
  test: {
    globals: true,
    include: ['src/**/*.test.ts']
  }
})
