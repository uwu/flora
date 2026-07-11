import { defineConfig } from '@hey-api/openapi-ts'

export default defineConfig({
  input: '../../openapi/flora.openapi.json',
  output: 'src/generated',
  plugins: ['@tanstack/react-query']
})
