import path from 'node:path'
import fs from 'node:fs'
import type { Plugin } from 'vite'
import UnoCSS from 'unocss/vite'
import vue from '@vitejs/plugin-vue'

function browserLogPlugin(): Plugin {
  return {
    name: 'flora-browser-logs',
    apply: 'serve',
    configureServer(server) {
      server.middlewares.use('/__debug/browser-log', (req, res) => {
        if (req.method !== 'POST') {
          res.statusCode = 405
          res.end('Method Not Allowed')
          return
        }

        let body = ''
        req.setEncoding('utf8')
        req.on('data', (chunk: string) => {
          body += chunk
        })
        req.on('end', () => {
          const logDir = path.resolve(__dirname, '../../.amp/in')
          fs.mkdirSync(logDir, { recursive: true })
          fs.appendFileSync(path.join(logDir, 'server.log'), `[browser] ${body}\n`)
          res.statusCode = 204
          res.end()
        })
      })
    }
  }
}

const unoPlugins = UnoCSS() as unknown as unknown[]
const plugins: unknown[] = [vue() as unknown, ...unoPlugins, browserLogPlugin() as unknown]

export default {
  plugins,
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  server: {
    allowedHosts: true,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      },
      '/__dev': {
        target: 'http://localhost:3000',
        changeOrigin: true
      }
    }
  }
}
