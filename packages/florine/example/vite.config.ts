import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite-plus'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    dedupe: ['react', 'react-dom', 'react/jsx-runtime']
  }
})
