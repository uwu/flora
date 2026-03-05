export type CliConfig = {
  apiUrl: string
  token?: string
}

export type DeployProjectConfig = {
  entry?: string
  root?: string
}

export const DEFAULT_API_URL = 'http://localhost:3000/api'
