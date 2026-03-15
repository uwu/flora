import tailwindcss from '@tailwindcss/vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import path from 'path'
import { defineConfig } from 'vite'
import babel from '@rolldown/plugin-babel'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const API_HOST = 'http://localhost:3000'
  console.log(`API: ${API_HOST}`)
  return {
    plugins: [react(), babel({ presets: [reactCompilerPreset()] }), tailwindcss()],
    optimizeDeps: {
      exclude: ['modern-monaco', 'modern-monaco/core']
    },
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
        'modern-monaco/core': path.resolve(__dirname, 'node_modules/modern-monaco/dist/core.mjs'),
        'monaco-themes/themes/themelist.json': path.resolve(
          __dirname,
          'node_modules/monaco-themes/themes/themelist.json'
        )
      }
    },

    ...(mode === 'development' && {
      server: {
        allowedHosts: true,
        proxy: {
          '/api': {
            target: API_HOST,
            changeOrigin: true
          }
        }
      }
    })
  }
})
