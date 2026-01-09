import path from "path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const API_HOST = 'http://localhost:3000'
  console.log(`API: ${API_HOST}`)
  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
        "monaco-themes/themes/themelist.json": path.resolve(__dirname, "node_modules/monaco-themes/themes/themelist.json"),
      },
    },

    ...(mode === 'development' && {
      server: {
        allowedHosts: true,
        proxy: {
          '/api': {
            target: API_HOST,
            changeOrigin: true,
          }
        }
      }
    })
  }
}
)
