import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'
import devtools from 'solid-devtools/vite'
import solidOxc from '@oxc-solid-js/vite'

export default defineConfig({
  plugins: [solidOxc(), tailwindcss()],
  server: {
    port: 3000
  },
  build: {
    target: 'esnext'
  }
})
