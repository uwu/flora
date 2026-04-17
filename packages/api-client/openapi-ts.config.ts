import { defineConfig } from '@hey-api/openapi-ts'

export default defineConfig({
  input: 'http://localhost:3000/api-docs/openapi.json',
  output: 'src/generated',
  plugins: ['@tanstack/react-query']
})
